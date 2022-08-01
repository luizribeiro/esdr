use crate::blocks::ESDRBlock;
use crate::ui::ESDRGraph;
use crate::ui::ESDRValueType;

use std::collections::HashMap;

use async_task::Task;
use egui_node_graph::NodeId;
use futuresdr::async_io;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::FlowgraphHandle;
use futuresdr::runtime::Pmt;
use futuresdr::runtime::Runtime;

#[allow(dead_code)]
pub struct Radio {
    task: Task<Result<Flowgraph, anyhow::Error>>,
    handle: FlowgraphHandle,
    node_id_to_block_id: HashMap<NodeId, usize>,
    message_id_for_field: HashMap<(NodeId, String), usize>,
}

pub fn start(graph: &ESDRGraph) -> Radio {
    let mut fg = Flowgraph::new();
    let mut node_id_to_block_id = HashMap::new();
    let mut message_id_for_field = HashMap::new();

    for node in &graph.nodes {
        let block = node.1.user_data.block_type.block(graph, node.1);
        for (name, input_id) in &node.1.inputs {
            let input = &graph.get_input(input_id.clone()).value;
            let allow_updates = match input {
                ESDRValueType::Scalar { allow_updates, .. } => *allow_updates,
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

    let (task, handle) = async_io::block_on(Runtime::new().start(fg));

    return Radio {
        task,
        handle,
        node_id_to_block_id,
        message_id_for_field,
    };
}

impl Radio {
    pub fn update_scalar(&mut self, node_id: NodeId, field: &str, value: f64) -> () {
        let port_id = self.message_id_for_field[&(node_id, field.to_string())];
        let block_id = self.node_id_to_block_id[&node_id];
        // FIXME this is super hacky
        let freq_offset = 250000.0;
        async_io::block_on(
            self.handle
                .call(block_id, port_id, Pmt::Double(value + freq_offset)),
        )
        .unwrap();
    }
}
