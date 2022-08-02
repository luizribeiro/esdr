use crate::params::Param;
use crate::params::ParamTrait;
use crate::ui::ESDRDataType;
use crate::ui::ESDRGraph;
use crate::ui::ESDRResponse;
use crate::ui::ESDRValueType;

use eframe::egui;
use egui_node_graph::InputParamKind;
use egui_node_graph::NodeId;

#[derive(Default, Clone, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct InputStream {
    name: String,
}

impl ParamTrait<()> for InputStream {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_input_param(
            node_id,
            self.name.clone(),
            ESDRDataType::Stream,
            ESDRValueType::InputStream {
                node_id,
                config: self,
            },
            InputParamKind::ConnectionOnly,
            true,
        );
    }

    fn widget(&mut self, ui: &mut egui::Ui, _node_id: NodeId, _value: ()) -> Vec<ESDRResponse> {
        ui.label(&self.name);
        vec![]
    }
}

impl InputStreamBuilder {
    pub fn build(&self) -> Param {
        Param::InputStream(self.build_impl().unwrap())
    }
}
