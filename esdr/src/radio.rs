use crate::ui::ESDRGraph;
use crate::ui::ESDRNodeData;
use crate::ui::ESDRNodeTemplate;
use crate::ui::ESDRValueType;

use std::collections::HashMap;

use async_task::Task;
use egui_node_graph::Node;
use egui_node_graph::NodeId;
use futuredsp::firdes;
use futuresdr::async_io;
use futuresdr::blocks::audio::AudioSink;
use futuresdr::blocks::Apply;
use futuresdr::blocks::FirBuilder;
use futuresdr::blocks::SoapySourceBuilder;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::FlowgraphHandle;
use futuresdr::runtime::Pmt;
use futuresdr::runtime::Runtime;

const RATE: f64 = 1000000.0;
const AUDIO_RATE: u32 = 48000;
const AUDIO_MULT: u32 = 5;

#[allow(dead_code)]
pub struct Radio {
    task: Task<Result<Flowgraph, anyhow::Error>>,
    handle: FlowgraphHandle,
    node_id_to_block_id: HashMap<NodeId, usize>,
    message_id_for_field: HashMap<(NodeId, String), usize>,
}

fn scalar_value(graph: &ESDRGraph, node: &Node<ESDRNodeData>, name: &str) -> f64 {
    let input_id = node.get_input(name).unwrap();
    let input = graph.get_input(input_id);
    match input.value {
        ESDRValueType::Scalar { value, .. } => value,
        _ => panic!("Unexpected value type"),
    }
}

fn build_block(graph: &ESDRGraph, node: &Node<ESDRNodeData>) -> Block {
    let freq_offset = RATE / 4.0;
    match node.user_data.template {
        ESDRNodeTemplate::SoapySDR => SoapySourceBuilder::new()
            .filter("")
            .freq(scalar_value(graph, node, "freq") + freq_offset)
            .sample_rate(RATE)
            .gain(scalar_value(graph, node, "gain"))
            .build(),
        ESDRNodeTemplate::Shift => {
            let mut last = Complex32::new(1.0, 0.0);
            let add = Complex32::from_polar(
                1.0,
                (2.0 * std::f64::consts::PI * freq_offset / RATE) as f32,
            );
            Apply::new(move |v: &Complex32| -> Complex32 {
                last *= add;
                last * v
            })
        }
        ESDRNodeTemplate::Resamp1 => {
            let interp = (AUDIO_RATE * AUDIO_MULT) as usize;
            let decim = RATE as usize;
            FirBuilder::new_resampling::<Complex32>(interp, decim)
        }
        ESDRNodeTemplate::FMDemodulator => {
            let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
            Apply::new(move |v: &Complex32| -> f32 {
                let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
                last = *v;
                arg
            })
        }
        ESDRNodeTemplate::Resamp2 => {
            let cutoff = scalar_value(graph, node, "cutoff") / (AUDIO_RATE * AUDIO_MULT) as f64;
            let transition =
                scalar_value(graph, node, "transition") / (AUDIO_RATE * AUDIO_MULT) as f64;
            let audio_filter_taps = firdes::kaiser::lowpass::<f32>(cutoff, transition, 0.1);
            FirBuilder::new_resampling_with_taps::<f32, f32, _>(
                1,
                AUDIO_MULT as usize,
                audio_filter_taps,
            )
        }
        ESDRNodeTemplate::AudioOutput => AudioSink::new(AUDIO_RATE, 1),
    }
}

pub fn start(graph: &ESDRGraph) -> Radio {
    let mut fg = Flowgraph::new();
    let mut node_id_to_block_id = HashMap::new();
    let mut message_id_for_field = HashMap::new();

    for node in &graph.nodes {
        let block = build_block(graph, node.1);
        for (name, input_id) in &node.1.inputs {
            let input = &graph.get_input(input_id.clone()).value;
            let allow_updates = match input {
                ESDRValueType::Scalar { allow_updates, .. } => *allow_updates,
                _ => false,
            };
            if allow_updates {
                let message_id = block
                    .message_input_name_to_id(name)
                    .expect(format!("Expected to find {}", name).as_str());
                message_id_for_field.insert((node.0, name.clone()), message_id);
            }
        }
        let block_id = fg.add_block(block);
        node_id_to_block_id.insert(node.0, block_id);
    }

    for (input_id, output_id) in &graph.connections {
        let input = graph.get_input(input_id);
        let output = graph.get_output(output_id.clone());
        let src = node_id_to_block_id[&output.node];
        let dest = node_id_to_block_id[&input.node];
        fg.connect_stream(src, "out", dest, "in").unwrap();
    }

    let (task, handle) = async_io::block_on(Runtime::new().start(fg));

    return Radio {
        task,
        handle,
        node_id_to_block_id,
        message_id_for_field,
    };
}

impl Radio {
    pub fn update_scalar(&mut self, node_id: NodeId, field: &str, value: f64) -> () {
        let port_id = self.message_id_for_field[&(node_id, field.to_string())];
        let block_id = self.node_id_to_block_id[&node_id];
        // FIXME this is super hacky
        let freq_offset = RATE / 4.0;
        async_io::block_on(
            self.handle
                .call(block_id, port_id, Pmt::Double(value + freq_offset)),
        )
        .unwrap();
    }
}
