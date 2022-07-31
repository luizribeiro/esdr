use std::borrow::Cow;

use eframe::egui::{self, DragValue};
use egui_node_graph::*;

use uuid::Uuid;

use crate::radio;

#[allow(dead_code)]
pub struct ESDRNodeData {
    uuid: Uuid,
    pub template: ESDRNodeTemplate,
}

#[derive(PartialEq, Eq)]
pub enum ESDRDataType {
    Stream,
    Scalar,
}

#[derive(Copy, Clone, Debug)]
pub enum ESDRValueType {
    Stream,
    Scalar { value: f64 },
}

#[derive(Clone, Copy)]
pub enum ESDRNodeTemplate {
    SoapySDR,
    Shift,
    Resamp1,
    FMDemodulator,
    Resamp2,
    AudioOutput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ESDRResponse {}

#[derive(Default)]
pub struct ESDRGraphState {}

impl DataTypeTrait<ESDRGraphState> for ESDRDataType {
    fn data_type_color(&self, _user_state: &ESDRGraphState) -> egui::Color32 {
        match self {
            ESDRDataType::Stream => egui::Color32::from_rgb(38, 109, 211),
            ESDRDataType::Scalar => egui::Color32::from_rgb(238, 207, 109),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            ESDRDataType::Stream => Cow::Borrowed("stream"),
            ESDRDataType::Scalar => Cow::Borrowed("scalar"),
        }
    }
}

impl NodeTemplateTrait for ESDRNodeTemplate {
    type NodeData = ESDRNodeData;
    type DataType = ESDRDataType;
    type ValueType = ESDRValueType;

    fn node_finder_label(&self) -> &str {
        match self {
            ESDRNodeTemplate::SoapySDR => "Soapy SDR",
            ESDRNodeTemplate::Shift => "Shift",
            ESDRNodeTemplate::Resamp1 => "Resamp 1",
            ESDRNodeTemplate::FMDemodulator => "FM Demodulator",
            ESDRNodeTemplate::Resamp2 => "Resamp 2",
            ESDRNodeTemplate::AudioOutput => "Audio Output",
        }
    }

    fn node_graph_label(&self) -> String {
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        ESDRNodeData {
            uuid: Uuid::new_v4(),
            template: self.clone(),
        }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        node_id: NodeId,
    ) {
        let scalar_value = |graph: &mut ESDRGraph, name: &str, value: f64| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                ESDRDataType::Scalar,
                ESDRValueType::Scalar { value },
                InputParamKind::ConstantOnly,
                true,
            );
        };

        let input_stream = |graph: &mut ESDRGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                ESDRDataType::Stream,
                ESDRValueType::Stream,
                InputParamKind::ConnectionOnly,
                true,
            );
        };
        let output_stream = |graph: &mut ESDRGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), ESDRDataType::Stream);
        };

        match self {
            ESDRNodeTemplate::SoapySDR => {
                output_stream(graph, "out");
                scalar_value(graph, "freq", 90900000.0);
                scalar_value(graph, "gain", 30.0);
            }
            ESDRNodeTemplate::Shift => {
                input_stream(graph, "in");
                output_stream(graph, "out");
            }
            ESDRNodeTemplate::Resamp1 => {
                input_stream(graph, "in");
                output_stream(graph, "out");
            }
            ESDRNodeTemplate::FMDemodulator => {
                input_stream(graph, "in");
                output_stream(graph, "out");
            }
            ESDRNodeTemplate::Resamp2 => {
                input_stream(graph, "in");
                output_stream(graph, "out");
            }
            ESDRNodeTemplate::AudioOutput => {
                input_stream(graph, "in");
            }
        }
    }
}

pub struct AllESDRNodeTemplates;
impl NodeTemplateIter for AllESDRNodeTemplates {
    type Item = ESDRNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            ESDRNodeTemplate::SoapySDR,
            ESDRNodeTemplate::Shift,
            ESDRNodeTemplate::Resamp1,
            ESDRNodeTemplate::FMDemodulator,
            ESDRNodeTemplate::Resamp2,
            ESDRNodeTemplate::AudioOutput,
        ]
    }
}

impl WidgetValueTrait for ESDRValueType {
    type Response = ESDRResponse;
    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<ESDRResponse> {
        match self {
            ESDRValueType::Stream => {
                ui.label(param_name);
            }
            ESDRValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
        }
        Vec::new()
    }
}

impl UserResponseTrait for ESDRResponse {}
impl NodeDataTrait for ESDRNodeData {
    type Response = ESDRResponse;
    type UserState = ESDRGraphState;
    type DataType = ESDRDataType;
    type ValueType = ESDRValueType;

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<ESDRNodeData, ESDRDataType, ESDRValueType>,
        _user_state: &Self::UserState,
    ) -> Vec<NodeResponse<ESDRResponse, ESDRNodeData>>
    where
        ESDRResponse: UserResponseTrait,
    {
        let responses = vec![];
        responses
    }
}

pub type ESDRGraph = Graph<ESDRNodeData, ESDRDataType, ESDRValueType>;
type ESDREditorState =
    GraphEditorState<ESDRNodeData, ESDRDataType, ESDRValueType, ESDRNodeTemplate, ESDRGraphState>;

pub struct ESDRApp {
    state: ESDREditorState,
    radio: Option<radio::Radio>,
}

impl Default for ESDRApp {
    fn default() -> Self {
        Self {
            state: GraphEditorState::new(1.0, ESDRGraphState::default()),
            radio: None,
        }
    }
}

impl eframe::App for ESDRApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.horizontal(|ui| {
                    if ui
                        .button(if self.radio.is_some() { "⏹" } else { "▶" })
                        .clicked()
                    {
                        if self.radio.is_none() {
                            self.radio = Some(radio::start(&self.state.graph));
                        } else {
                            self.radio = None;
                        }
                    }
                });
            });
        });
        let _graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.state.draw_graph_editor(ui, AllESDRNodeTemplates)
            })
            .inner;
    }
}

pub fn run() -> ! {
    eframe::run_native(
        "eSDR",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(ESDRApp::default())),
    );
}
