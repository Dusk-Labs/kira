use serde_json::Value;
use std::collections::HashMap;

use super::workflow::*;
use crate::model::tabs::project::graph::Graph;
use crate::model::tabs::project::graph::NodeInstance;
use crate::model::tabs::project::graph::NodeType;
use crate::model::tabs::project::Node as AvailableNode;

type IdLut = HashMap<usize, usize>;

#[allow(dead_code)]
pub struct WorkflowBuilder {
    node_id_lut: IdLut,
    workflow: Workflow,
    available_nodes: HashMap<NodeType, AvailableNode>,
}

#[allow(dead_code)]
impl WorkflowBuilder {
    pub fn new(available_nodes: HashMap<NodeType, AvailableNode>) -> Self {
        Self {
            node_id_lut: IdLut::new(),
            workflow: Workflow::new(),
            available_nodes,
        }
    }

    pub fn into_workflow(mut self, graph: &Graph) -> Workflow {
        for link in graph.get_links() {
            let Some((src, dst)) = graph
                .get_node(link.src_node)
                .zip(graph.get_node(link.dst_node))
            else {
                continue;
            };

            let Some((src_widgets_values, dst_widgets_values)) = graph
                .get_state(link.src_node)
                .zip(graph.get_state(link.dst_node))
            else {
                continue;
            };

            let mut src_widgets_values = src_widgets_values
                .iter()
                .filter_map(|(_, value)| value.state())
                .collect::<Vec<_>>();

            let mut dst_widgets_values = dst_widgets_values
                .iter()
                .filter_map(|(_, value)| value.state())
                .collect::<Vec<_>>();

            // FIXME: Patch for comfyui#3419 where ksampler control_after_generate value is
            // expected but not defined in the input list
            [
                (&src.ty.0, &mut src_widgets_values),
                (&dst.ty.0, &mut dst_widgets_values),
            ]
            .iter_mut()
            .filter(|(ty, _)| *ty == "KSampler")
            .for_each(|(_, values)| values.insert(1, "fixed".into()));

            let Some(src_id) = self.create_or_get_node(src, link.src_node, src_widgets_values)
            else {
                continue;
            };

            let Some(dst_id) = self.create_or_get_node(dst, link.dst_node, dst_widgets_values)
            else {
                continue;
            };

            self.workflow
                .link(src_id, dst_id, link.src_slot, link.dst_slot)
                .unwrap();
        }

        self.workflow
    }

    fn create_or_get_node(
        &mut self,
        node: &NodeInstance,
        idx: usize,
        widgets_values: Vec<Value>,
    ) -> Option<usize> {
        if let Some(node_id) = self.node_id_lut.get(&idx) {
            return Some(*node_id);
        }

        let meta = self.available_nodes.get(&node.ty)?;
        let (x, y) = node.pos;

        // inputs would only be links
        // fields dont count as inputs??
        let inputs = meta
            .inputs
            .iter()
            .map(|(name, ty)| WorkflowNodeInput {
                name: name.clone(),
                ty: ty.0.clone(),
                link: 0,
            })
            .collect::<Vec<_>>();

        let outputs = meta
            .outputs
            .iter()
            .enumerate()
            .map(|(idx, (name, ty))| WorkflowNodeOutput {
                name: name.clone(),
                ty: ty.0.clone(),
                links: vec![],
                slot_index: idx,
            })
            .collect::<Vec<_>>();

        let conv_node = WorkflowNode {
            id: 0,
            ty: node.ty.0.clone(),
            pos: (x as _, y as _),
            order: 1,
            mode: 0,
            inputs,
            outputs,
            properties: Default::default(),
            widgets_values,
        };

        let id = self.workflow.add_node(conv_node);

        self.node_id_lut.insert(idx, id);

        Some(id)
    }
}
