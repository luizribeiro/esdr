use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockType;
use crate::params::input_stream::InputStream;
use crate::params::scalar::ScalarParam;
use crate::params::Param;
use crate::params::ParamTrait;
use crate::radio;

use std::borrow::Cow;

use eframe::egui;
use egui_node_graph::*;
use strum::IntoEnumIterator;
use uuid::Uuid;

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
    InputStream {
        node_id: NodeId,
        config: InputStream,
    },
    Scalar {
        node_id: NodeId,
        value: f64,
        config: ScalarParam,
    },
}

#[derive(Clone, Debug)]
pub struct UpdateScalarPayload {
    pub node_id: NodeId,
    pub field: String,
    pub value: f64,
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
        for param in self.params() {
            // this is needed because enum_dispatch doesn't work with traits that have
            // associated types. see https://gitlab.com/antonok/enum_dispatch/-/issues/50
            match param {
                Param::Scalar(p) => p.add_param(graph, node_id),
                Param::InputStream(p) => p.add_param(graph, node_id),
                Param::OutputStream(p) => p.add_param(graph, node_id),
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
    fn value_widget(&mut self, _param_name: &str, ui: &mut egui::Ui) -> Vec<ESDRResponse> {
        let mut responses = vec![];
        match self {
            ESDRValueType::InputStream {
                node_id, config, ..
            } => {
                responses.append(&mut config.widget(ui, *node_id, ()));
            }
            ESDRValueType::Scalar {
                node_id,
                value,
                config,
            } => {
                responses.append(&mut config.widget(ui, *node_id, value));
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
