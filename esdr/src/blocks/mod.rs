use crate::param::Param;
use crate::ui::ESDRGraph;
use crate::ui::ESDRNodeData;
use crate::ui::ESDRValueType;

use egui_node_graph::Node;
use futuresdr::runtime::Block;
use strum_macros::EnumIter;

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
        match input.value {
            ESDRValueType::Scalar { value, .. } => value,
            _ => panic!("Unexpected value type"),
        }
    }
}

mod audio_output;
mod fmdemod;
mod resamp1;
mod resamp2;
mod shift;
mod soapysdr;

#[enum_dispatch(ESDRBlock)]
#[derive(Clone, Copy, EnumIter)]
pub enum ESDRBlockType {
    SoapySDR(self::soapysdr::SoapySDRBlock),
    Shift(self::shift::ShiftBlock),
    Resamp1(self::resamp1::Resamp1Block),
    FMDemodulator(self::fmdemod::FMDemodulatorBlock),
    Resamp2(self::resamp2::Resamp2Block),
    AudioOutput(self::audio_output::AudioOutputBlock),
}
