use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockInput;
use crate::ui::ESDRGraph;
use crate::ui::ESDRValueType;

use std::collections::HashMap;

use async_task::Task;
use egui_node_graph::NodeId;
use futuresdr::async_io;
use futuresdr::runtime::scheduler::SmolScheduler;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::FlowgraphHandle;
use futuresdr::runtime::Pmt;
use futuresdr::runtime::Runtime;

#[allow(dead_code)]
pub struct Radio {
    running: Option<(
        Task<Result<Flowgraph, anyhow::Error>>,
        FlowgraphHandle,
        Runtime<SmolScheduler>,
    )>,
    node_id_to_block_id: HashMap<NodeId, usize>,
    message_id_for_field: HashMap<(NodeId, String), usize>,
}

pub fn start(graph: &ESDRGraph) -> Radio {
    let mut fg = Flowgraph::new();
    let mut node_id_to_block_id = HashMap::new();
    let mut message_id_for_field = HashMap::new();

    for node in &graph.nodes {
        let input = ESDRBlockInput::new(&graph, &node.1);
        let block = node.1.user_data.block_type.block(input);
        for (name, input_id) in &node.1.inputs {
            let input = &graph.get_input(input_id.clone()).value;
            let allow_updates = match input {
                ESDRValueType::Scalar { config, .. } => config.allow_updates,
                _ => false,
            };
            if allow_updates {
                let message_id = block
                    .message_input_name_to_id(name)
                    .expect(format!("Expected to find {}", name).as_str());
                message_id_for_field.insert((node.0, name.clone()), message_id);
            }
        }
        let block_id = fg.add_block(block);
        node_id_to_block_id.insert(node.0, block_id);
    }

    for (input_id, output_id) in &graph.connections {
        let input = graph.get_input(input_id);
        let output = graph.get_output(output_id.clone());
        let src = node_id_to_block_id[&output.node];
        let dest = node_id_to_block_id[&input.node];
        fg.connect_stream(src, "out", dest, "in").unwrap();
    }

    // TODO: turn this into an async function instead of blocking
    let runtime = Runtime::new();
    let (task, handle) = async_io::block_on(runtime.start(fg));

    return Radio {
        running: Some((task, handle, runtime)),
        node_id_to_block_id,
        message_id_for_field,
    };
}

impl Radio {
    pub fn stop(&mut self) {
        // TODO: turn this into an async function
        if let Some((task, mut handle, _)) = self.running.take() {
            async_io::block_on(async move {
                handle.terminate().await.unwrap();
                task.await.unwrap();
            });
        }
    }

    pub fn update_scalar(&mut self, node_id: NodeId, field: &str, value: f64) {
        if let Some(ref mut running) = self.running {
            let port_id = self.message_id_for_field[&(node_id, field.to_string())];
            let block_id = self.node_id_to_block_id[&node_id];
            // FIXME this is super hacky
            let freq_offset = 250000.0;
            async_io::block_on(
                running
                    .1
                    .call(block_id, port_id, Pmt::Double(value + freq_offset)),
            )
            .unwrap();
        }
    }
}
