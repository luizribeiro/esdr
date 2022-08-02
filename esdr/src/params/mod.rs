use crate::params::input_stream::InputStreamBuilder;
use crate::params::output_stream::OutputStreamBuilder;
use crate::params::scalar::ScalarParamBuilder;
use crate::ui::ESDRGraph;
use crate::ui::ESDRResponse;

use eframe::egui;
use egui_node_graph::NodeId;

pub mod input_stream;
pub mod output_stream;
pub mod scalar;

#[derive(Clone, Debug)]
pub enum Param {
    Scalar(self::scalar::ScalarParam),
    InputStream(self::input_stream::InputStream),
    OutputStream(self::output_stream::OutputStream),
}

pub trait ParamTrait<T> {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> ();
    fn widget(&mut self, ui: &mut egui::Ui, node_id: NodeId, value: T) -> Vec<ESDRResponse>;
}

impl Param {
    pub fn input_stream(name: &str) -> InputStreamBuilder {
        InputStreamBuilder::default().name(name).clone()
    }

    pub fn output_stream(name: &str) -> OutputStreamBuilder {
        OutputStreamBuilder::default().name(name).clone()
    }

    pub fn scalar(name: &str) -> ScalarParamBuilder {
        ScalarParamBuilder::default().name(name).clone()
    }
}
