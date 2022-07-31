use crate::ui::ESDRGraph;
use crate::ui::ESDRNodeData;
use crate::ui::ESDRNodeTemplate;
use crate::ui::ESDRValueType;

use std::collections::HashMap;

use egui_node_graph::Node;
use futuredsp::firdes;
use futuresdr::async_io;
use futuresdr::blocks::audio::AudioSink;
use futuresdr::blocks::Apply;
use futuresdr::blocks::FirBuilder;
use futuresdr::blocks::SoapySourceBuilder;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

const RATE: f64 = 1000000.0;
const AUDIO_RATE: u32 = 48000;
const AUDIO_MULT: u32 = 5;

#[allow(dead_code)]
pub struct Radio {
    task: async_task::Task<Result<Flowgraph, anyhow::Error>>,
}

fn scalar_value(graph: &ESDRGraph, node: &Node<ESDRNodeData>, name: &str) -> f64 {
    let input_id = node.get_input(name).unwrap();
    let input = graph.get_input(input_id);
    match input.value {
        ESDRValueType::Scalar { value } => value,
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
            let cutoff = 2_000.0 / (AUDIO_RATE * AUDIO_MULT) as f64;
            let transition = 10_000.0 / (AUDIO_RATE * AUDIO_MULT) as f64;
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

fn build_flowgraph(graph: &ESDRGraph) -> Flowgraph {
    let mut fg = Flowgraph::new();
    let mut node_to_blocks = HashMap::new();

    for node in &graph.nodes {
        let block = build_block(graph, node.1);
        let block_id = fg.add_block(block);
        node_to_blocks.insert(node.0, block_id);
    }

    for (input_id, output_id) in &graph.connections {
        let input = graph.get_input(input_id);
        let output = graph.get_output(output_id.clone());
        let src = node_to_blocks[&output.node];
        let dest = node_to_blocks[&input.node];
        fg.connect_stream(src, "out", dest, "in").unwrap();
    }

    return fg;
}

pub fn start(graph: &ESDRGraph) -> Radio {
    let fg = build_flowgraph(graph);
    let (res, mut _handle) = async_io::block_on(Runtime::new().start(fg));
    return Radio { task: res };
}
