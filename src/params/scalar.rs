use crate::params::Param;
use crate::params::ParamTrait;
use crate::ui::ESDRDataType;
use crate::ui::ESDRGraph;
use crate::ui::ESDRResponse;
use crate::ui::ESDRValueType;
use crate::ui::UpdateScalarPayload;

use eframe::egui::{self, DragValue};
use egui_node_graph::InputParamKind;
use egui_node_graph::NodeId;

#[derive(Default, Clone, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct ScalarParam {
    pub name: String,
    #[builder(default = "0.0")]
    pub initial_value: f64,
    #[builder(default = "false")]
    pub allow_updates: bool,
}

impl ParamTrait<&mut f64> for ScalarParam {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_input_param(
            node_id,
            self.name.clone(),
            ESDRDataType::Scalar,
            ESDRValueType::Scalar {
                node_id,
                value: self.initial_value,
                config: self,
            },
            InputParamKind::ConstantOnly,
            true,
        );
    }

    fn widget(&mut self, ui: &mut egui::Ui, node_id: NodeId, value: &mut f64) -> Vec<ESDRResponse> {
        let mut responses = vec![];
        ui.horizontal(|ui| {
            ui.label(&self.name);
            if ui.add(DragValue::new(value)).changed() {
                responses.push(ESDRResponse::UpdateScalar(UpdateScalarPayload {
                    node_id,
                    field: self.name.to_string(),
                    value: *value,
                }));
            }
        });
        responses
    }
}

impl ScalarParamBuilder {
    pub fn build(&self) -> Param {
        Param::Scalar(self.build_impl().unwrap())
    }
}
