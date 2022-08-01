use eframe::egui::{self, DragValue};
use egui_node_graph::InputParamKind;
use egui_node_graph::NodeId;

use crate::ui::ESDRDataType;
use crate::ui::ESDRGraph;
use crate::ui::ESDRResponse;
use crate::ui::ESDRValueType;
use crate::ui::UpdateScalarPayload;

#[derive(Clone, Debug)]
#[enum_dispatch(ParamTrait)]
pub enum Param {
    Scalar(ScalarParam),
    InputStream(InputStream),
    OutputStream(OutputStream),
}

#[enum_dispatch]
pub trait ParamTrait {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> ();
    fn widget(&mut self, ui: &mut egui::Ui, node_id: NodeId) -> Vec<ESDRResponse>;
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

#[derive(Default, Clone, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct ScalarParam {
    pub name: String,
    #[builder(default = "0.0", setter(prefix = "initial"))]
    pub value: f64,
    #[builder(default = "false")]
    pub allow_updates: bool,
}

impl ParamTrait for ScalarParam {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_input_param(
            node_id,
            self.name.clone(),
            ESDRDataType::Scalar,
            ESDRValueType::Scalar {
                node_id,
                config: self.clone(),
            },
            InputParamKind::ConstantOnly,
            true,
        );
    }

    fn widget(&mut self, ui: &mut egui::Ui, node_id: NodeId) -> Vec<ESDRResponse> {
        let mut responses = vec![];
        ui.horizontal(|ui| {
            ui.label(&self.name);
            if ui.add(DragValue::new(&mut self.value)).changed() {
                responses.push(ESDRResponse::UpdateScalar(UpdateScalarPayload {
                    node_id,
                    field: self.name.to_string(),
                    value: self.value,
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

#[derive(Default, Clone, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct InputStream {
    name: String,
}

impl ParamTrait for InputStream {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_input_param(
            node_id,
            self.name.clone(),
            ESDRDataType::Stream,
            ESDRValueType::Stream,
            InputParamKind::ConnectionOnly,
            true,
        );
    }

    fn widget(&mut self, ui: &mut egui::Ui, _node_id: NodeId) -> Vec<ESDRResponse> {
        ui.label(&self.name);
        vec![]
    }
}

impl InputStreamBuilder {
    pub fn build(&self) -> Param {
        Param::InputStream(self.build_impl().unwrap())
    }
}

#[derive(Default, Clone, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct OutputStream {
    name: String,
}

impl ParamTrait for OutputStream {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_output_param(node_id, self.name.clone(), ESDRDataType::Stream);
    }

    fn widget(&mut self, ui: &mut egui::Ui, _node_id: NodeId) -> Vec<ESDRResponse> {
        ui.label(&self.name);
        vec![]
    }
}

impl OutputStreamBuilder {
    pub fn build(&self) -> Param {
        Param::OutputStream(self.build_impl().unwrap())
    }
}
