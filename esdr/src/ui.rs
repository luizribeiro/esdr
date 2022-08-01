use std::borrow::Cow;

use eframe::egui::{self, DragValue};
use egui_node_graph::*;
use futuresdr::runtime::Block;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::blocks::*;
use crate::radio;

#[allow(dead_code)]
pub struct ESDRNodeData {
    uuid: Uuid,
    pub block_type: ESDRBlockType,
}

#[derive(PartialEq, Eq)]
pub enum ESDRDataType {
    Stream,
    Scalar,
}

#[derive(Clone, Debug)]
pub enum ESDRValueType {
    Stream,
    Scalar {
        node_id: NodeId,
        field: String,
        value: f64,
        allow_updates: bool,
    },
}

#[enum_dispatch(ESDRBlock)]
#[derive(Clone, Copy, EnumIter)]
pub enum ESDRBlockType {
    SoapySDR(SoapySDRBlock),
    Shift(ShiftBlock),
    Resamp1(Resamp1Block),
    FMDemodulator(FMDemodulatorBlock),
    Resamp2(Resamp2Block),
    AudioOutput(AudioOutputBlock),
}

#[derive(Clone, Debug)]
pub struct UpdateScalarPayload {
    node_id: NodeId,
    field: String,
    value: f64,
}

#[derive(Clone, Debug)]
pub enum ESDRResponse {
    UpdateScalar(UpdateScalarPayload),
}

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

impl NodeTemplateTrait for ESDRBlockType {
    type NodeData = ESDRNodeData;
    type DataType = ESDRDataType;
    type ValueType = ESDRValueType;

    fn node_finder_label(&self) -> &str {
        self.name()
    }

    fn node_graph_label(&self) -> String {
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        ESDRNodeData {
            uuid: Uuid::new_v4(),
            block_type: self.clone(),
        }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        node_id: NodeId,
    ) {
        let scalar_value = |graph: &mut ESDRGraph, name: &str, value: f64, allow_updates: bool| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                ESDRDataType::Scalar,
                ESDRValueType::Scalar {
                    node_id,
                    field: name.to_string(),
                    value,
                    allow_updates,
                },
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
            ESDRBlockType::SoapySDR(_) => {
                output_stream(graph, "out");
                scalar_value(graph, "freq", 90900000.0, true);
                scalar_value(graph, "gain", 30.0, false);
            }
            ESDRBlockType::Shift(_) => {
                input_stream(graph, "in");
                output_stream(graph, "out");
            }
            ESDRBlockType::Resamp1(_) => {
                input_stream(graph, "in");
                output_stream(graph, "out");
            }
            ESDRBlockType::FMDemodulator(_) => {
                input_stream(graph, "in");
                output_stream(graph, "out");
            }
            ESDRBlockType::Resamp2(_) => {
                input_stream(graph, "in");
                scalar_value(graph, "cutoff", 2000.0, false);
                scalar_value(graph, "transition", 10000.0, false);
                output_stream(graph, "out");
            }
            ESDRBlockType::AudioOutput(_) => {
                input_stream(graph, "in");
            }
        }
    }
}

pub struct AllESDRBlockTypes;
impl NodeTemplateIter for AllESDRBlockTypes {
    type Item = ESDRBlockType;

    fn all_kinds(&self) -> Vec<Self::Item> {
        ESDRBlockType::iter().collect()
    }
}

impl WidgetValueTrait for ESDRValueType {
    type Response = ESDRResponse;
    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<ESDRResponse> {
        let mut responses = vec![];
        match self {
            ESDRValueType::Stream => {
                ui.label(param_name);
            }
            ESDRValueType::Scalar {
                node_id,
                field,
                value,
                ..
            } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    if ui.add(DragValue::new(value)).changed() {
                        responses.push(ESDRResponse::UpdateScalar(UpdateScalarPayload {
                            node_id: *node_id,
                            field: field.to_string(),
                            value: *value,
                        }));
                    }
                });
            }
        }
        responses
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
    GraphEditorState<ESDRNodeData, ESDRDataType, ESDRValueType, ESDRBlockType, ESDRGraphState>;

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
        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.state.draw_graph_editor(ui, AllESDRBlockTypes)
            })
            .inner;
        for response in graph_response.node_responses {
            if let NodeResponse::User(user_event) = response {
                match user_event {
                    ESDRResponse::UpdateScalar(ev) => {
                        if let Some(radio) = &mut self.radio {
                            radio.update_scalar(ev.node_id, &ev.field, ev.value);
                        }
                    }
                }
            }
        }
    }
}

pub fn run() -> ! {
    eframe::run_native(
        "eSDR",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(ESDRApp::default())),
    );
}
