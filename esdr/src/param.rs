use egui_node_graph::InputParamKind;
use egui_node_graph::NodeId;

use crate::ui::ESDRDataType;
use crate::ui::ESDRGraph;
use crate::ui::ESDRValueType;

#[enum_dispatch(ParamTrait)]
pub enum Param {
    Scalar(ScalarParam),
    InputStream(InputStream),
    OutputStream(OutputStream),
}

#[enum_dispatch]
pub trait ParamTrait {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> ();
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

#[derive(Default, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct ScalarParam {
    name: String,
    #[builder(default = "0.0")]
    initial_value: f64,
    #[builder(default = "false")]
    allow_updates: bool,
}

impl ParamTrait for ScalarParam {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_input_param(
            node_id,
            self.name.clone(),
            ESDRDataType::Scalar,
            ESDRValueType::Scalar {
                node_id,
                field: self.name.clone(),
                value: self.initial_value,
                allow_updates: self.allow_updates,
            },
            InputParamKind::ConstantOnly,
            true,
        );
    }
}

impl ScalarParamBuilder {
    pub fn build(&self) -> Param {
        Param::Scalar(self.build_impl().unwrap())
    }
}

#[derive(Default, Builder, Debug)]
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
}

impl InputStreamBuilder {
    pub fn build(&self) -> Param {
        Param::InputStream(self.build_impl().unwrap())
    }
}

#[derive(Default, Builder, Debug)]
#[builder(public, setter(into), build_fn(private, name = "build_impl"))]
pub struct OutputStream {
    name: String,
}

impl ParamTrait for OutputStream {
    fn add_param(self, graph: &mut ESDRGraph, node_id: NodeId) -> () {
        graph.add_output_param(node_id, self.name.clone(), ESDRDataType::Stream);
    }
}

impl OutputStreamBuilder {
    pub fn build(&self) -> Param {
        Param::OutputStream(self.build_impl().unwrap())
    }
}
