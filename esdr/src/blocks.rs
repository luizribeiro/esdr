use crate::param::Param;
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
    fn block(self, input: ESDRBlockInput) -> Block;
    fn params(self) -> Vec<Param>;
}

pub struct ESDRBlockInput<'a> {
    graph: &'a ESDRGraph,
    node: &'a Node<ESDRNodeData>,
}

impl ESDRBlockInput<'_> {
    pub fn new<'a>(graph: &'a ESDRGraph, node: &'a Node<ESDRNodeData>) -> ESDRBlockInput<'a> {
        ESDRBlockInput { graph, node }
    }

    pub fn scalar(&self, name: &str) -> f64 {
        let input_id = self.node.get_input(name).unwrap();
        let input = self.graph.get_input(input_id);
        match &input.value {
            ESDRValueType::Scalar { config, .. } => config.value,
            _ => panic!("Unexpected value type"),
        }
    }
}

const RATE: f64 = 1000000.0;
const FREQ_OFFSET: f64 = 250000.0;
const AUDIO_RATE: u32 = 48000;
const AUDIO_MULT: u32 = 5;

#[derive(Clone, Copy, Default)]
pub struct SoapySDRBlock {}
impl ESDRBlock for SoapySDRBlock {
    fn name(self) -> &'static str {
        "Soapy SDR"
    }

    fn block(self, input: ESDRBlockInput) -> Block {
        SoapySourceBuilder::new()
            .filter("")
            .freq(input.scalar("freq") + FREQ_OFFSET)
            .sample_rate(RATE)
            .gain(input.scalar("gain"))
            .build()
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::output_stream("out").build(),
            Param::scalar("freq")
                .initial_value(90900000.0)
                .allow_updates(true)
                .build(),
            Param::scalar("gain").initial_value(30.0).build(),
        ]
    }
}

#[derive(Clone, Copy, Default)]
pub struct ShiftBlock {}
impl ESDRBlock for ShiftBlock {
    fn name(self) -> &'static str {
        "Shift"
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
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

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::output_stream("out").build(),
        ]
    }
}

#[derive(Clone, Copy, Default)]
pub struct Resamp1Block {}
impl ESDRBlock for Resamp1Block {
    fn name(self) -> &'static str {
        "Resamp 1"
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
        let interp = (AUDIO_RATE * AUDIO_MULT) as usize;
        let decim = RATE as usize;
        FirBuilder::new_resampling::<Complex32>(interp, decim)
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::output_stream("out").build(),
        ]
    }
}

#[derive(Clone, Copy, Default)]
pub struct FMDemodulatorBlock {}
impl ESDRBlock for FMDemodulatorBlock {
    fn name(self) -> &'static str {
        "FM Demodulator"
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
        let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
        Apply::new(move |v: &Complex32| -> f32 {
            let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
            last = *v;
            arg
        })
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::output_stream("out").build(),
        ]
    }
}

#[derive(Clone, Copy, Default)]
pub struct Resamp2Block {}
impl ESDRBlock for Resamp2Block {
    fn name(self) -> &'static str {
        "Resamp 2"
    }

    fn block(self, input: ESDRBlockInput) -> Block {
        let cutoff = input.scalar("cutoff") / (AUDIO_RATE * AUDIO_MULT) as f64;
        let transition = input.scalar("transition") / (AUDIO_RATE * AUDIO_MULT) as f64;
        let audio_filter_taps = firdes::kaiser::lowpass::<f32>(cutoff, transition, 0.1);
        FirBuilder::new_resampling_with_taps::<f32, f32, _>(
            1,
            AUDIO_MULT as usize,
            audio_filter_taps,
        )
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::scalar("cutoff").initial_value(2000.0).build(),
            Param::scalar("transition").initial_value(10000.0).build(),
            Param::output_stream("out").build(),
        ]
    }
}

#[derive(Clone, Copy, Default)]
pub struct AudioOutputBlock {}
impl ESDRBlock for AudioOutputBlock {
    fn name(self) -> &'static str {
        "Audio Output"
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
        AudioSink::new(AUDIO_RATE, 1)
    }

    fn params(self) -> Vec<Param> {
        vec![Param::input_stream("in").build()]
    }
}
