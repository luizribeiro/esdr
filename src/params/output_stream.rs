use crate::params::Param;
use crate::params::ParamTrait;
use crate::ui::ESDRDataType;
use crate::ui::ESDRGraph;
use crate::ui::ESDRResponse;

use eframe::egui;
use egui_node_graph::NodeId;

#[derive(Default, Clone, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct OutputStream {
    name: String,
}

impl ParamTrait<()> for OutputStream {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_output_param(node_id, self.name.clone(), ESDRDataType::Stream);
    }

    fn widget(&mut self, ui: &mut egui::Ui, _node_id: NodeId, _value: ()) -> Vec<ESDRResponse> {
        ui.label(&self.name);
        vec![]
    }
}

impl OutputStreamBuilder {
    pub fn build(&self) -> Param {
        Param::OutputStream(self.build_impl().unwrap())
    }
}
