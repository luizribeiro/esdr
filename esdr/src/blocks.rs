use crate::ui::ESDRDataType;
use crate::ui::ESDRGraph;
use crate::ui::ESDRNodeData;
use crate::ui::ESDRValueType;

use egui_node_graph::Node;
use futuredsp::firdes;
use futuresdr::blocks::audio::AudioSink;
use futuresdr::blocks::Apply;
use futuresdr::blocks::FirBuilder;
use futuresdr::blocks::SoapySourceBuilder;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;

#[enum_dispatch]
pub trait ESDRBlock {
    fn name(self) -> &'static str;
    fn block(self, graph: &ESDRGraph, node: &Node<ESDRNodeData>) -> Block;
    fn params(self) -> Vec<ESDRBlockParam>;
}

pub struct ESDRBlockParam {
    name: String,
    data_type: ESDRDataType,
}

const RATE: f64 = 1000000.0;
const FREQ_OFFSET: f64 = 250000.0;
const AUDIO_RATE: u32 = 48000;
const AUDIO_MULT: u32 = 5;

fn scalar_value(graph: &ESDRGraph, node: &Node<ESDRNodeData>, name: &str) -> f64 {
    let input_id = node.get_input(name).unwrap();
    let input = graph.get_input(input_id);
    match input.value {
        ESDRValueType::Scalar { value, .. } => value,
        _ => panic!("Unexpected value type"),
    }
}

#[derive(Clone, Copy, Default)]
pub struct SoapySDRBlock {}
impl ESDRBlock for SoapySDRBlock {
    fn name(self) -> &'static str {
        "Soapy SDR"
    }

    fn block(self, graph: &ESDRGraph, node: &Node<ESDRNodeData>) -> Block {
        SoapySourceBuilder::new()
            .filter("")
            .freq(scalar_value(graph, node, "freq") + FREQ_OFFSET)
            .sample_rate(RATE)
            .gain(scalar_value(graph, node, "gain"))
            .build()
    }

    fn params(self) -> Vec<ESDRBlockParam> {
        // TODO
        vec![]
    }
}

#[derive(Clone, Copy, Default)]
pub struct ShiftBlock {}
impl ESDRBlock for ShiftBlock {
    fn name(self) -> &'static str {
        "Shift"
    }

    fn block(self, _graph: &ESDRGraph, _node: &Node<ESDRNodeData>) -> Block {
        let mut last = Complex32::new(1.0, 0.0);
        let add = Complex32::from_polar(
            1.0,
            (2.0 * std::f64::consts::PI * FREQ_OFFSET / RATE) as f32,
        );
        Apply::new(move |v: &Complex32| -> Complex32 {
            last *= add;
            last * v
        })
    }

    fn params(self) -> Vec<ESDRBlockParam> {
        // TODO
        vec![]
    }
}

#[derive(Clone, Copy, Default)]
pub struct Resamp1Block {}
impl ESDRBlock for Resamp1Block {
    fn name(self) -> &'static str {
        "Resamp 1"
    }

    fn block(self, _graph: &ESDRGraph, _node: &Node<ESDRNodeData>) -> Block {
        let interp = (AUDIO_RATE * AUDIO_MULT) as usize;
        let decim = RATE as usize;
        FirBuilder::new_resampling::<Complex32>(interp, decim)
    }

    fn params(self) -> Vec<ESDRBlockParam> {
        // TODO
        vec![]
    }
}

#[derive(Clone, Copy, Default)]
pub struct FMDemodulatorBlock {}
impl ESDRBlock for FMDemodulatorBlock {
    fn name(self) -> &'static str {
        "FM Demodulator"
    }

    fn block(self, _graph: &ESDRGraph, _node: &Node<ESDRNodeData>) -> Block {
        let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
        Apply::new(move |v: &Complex32| -> f32 {
            let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
            last = *v;
            arg
        })
    }

    fn params(self) -> Vec<ESDRBlockParam> {
        // TODO
        vec![]
    }
}

#[derive(Clone, Copy, Default)]
pub struct Resamp2Block {}
impl ESDRBlock for Resamp2Block {
    fn name(self) -> &'static str {
        "Resamp 2"
    }

    fn block(self, graph: &ESDRGraph, node: &Node<ESDRNodeData>) -> Block {
        let cutoff = scalar_value(graph, node, "cutoff") / (AUDIO_RATE * AUDIO_MULT) as f64;
        let transition = scalar_value(graph, node, "transition") / (AUDIO_RATE * AUDIO_MULT) as f64;
        let audio_filter_taps = firdes::kaiser::lowpass::<f32>(cutoff, transition, 0.1);
        FirBuilder::new_resampling_with_taps::<f32, f32, _>(
            1,
            AUDIO_MULT as usize,
            audio_filter_taps,
        )
    }

    fn params(self) -> Vec<ESDRBlockParam> {
        // TODO
        vec![]
    }
}

#[derive(Clone, Copy, Default)]
pub struct AudioOutputBlock {}
impl ESDRBlock for AudioOutputBlock {
    fn name(self) -> &'static str {
        "Audio Output"
    }

    fn block(self, _graph: &ESDRGraph, _node: &Node<ESDRNodeData>) -> Block {
        AudioSink::new(AUDIO_RATE, 1)
    }

    fn params(self) -> Vec<ESDRBlockParam> {
        // TODO
        vec![]
    }
}
