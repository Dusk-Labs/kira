use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::model::backend::node::NodeField;
use crate::model::tabs::project::graph::Graph;
use crate::model::tabs::project::graph::NodeType;
use crate::model::tabs::project::Node as AvailableNode;

pub type NodeId = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkflowPrompt {
    pub client_id: String,
    pub prompt: HashMap<NodeId, PromptNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<ExtraData>,
}

impl WorkflowPrompt {
    pub fn new() -> Self {
        Self {
            client_id: "f9e9494bb05849738d26b3b914e3eec2".into(),
            prompt: HashMap::new(),
            extra_data: None,
        }
    }

    pub fn from_graph(available_nodes: &HashMap<NodeType, AvailableNode>, graph: &Graph) -> Self {
        let mut wf = Self::new();

        for link in graph.get_links() {
            let Some((src, dst)) = graph
                .get_node(link.src_node)
                .zip(graph.get_node(link.dst_node))
            else {
                continue;
            };

            let Some(src_widgets_values) = graph.get_state(link.src_node) else {
                continue;
            };

            let Some(dst_widgets_values) = graph.get_state(link.dst_node) else {
                continue;
            };

            let dst_node =
                wf.create_or_get_node(dst.ty.0.clone(), link.dst_node, dst_widgets_values);

            let Some(node_template) = available_nodes.get(&dst.ty) else {
                continue;
            };

            let (field, _ty) = dbg!(&node_template.inputs)
                .get(dbg!(link.dst_slot))
                .expect("cant find input slot on inputting node");

            let vlink = PromptInputValue::Link {
                input_idx: link.src_slot,
                node_id: link.src_node.to_string(),
            };

            dst_node.inputs.insert(field.clone(), vlink);

            wf.create_or_get_node(src.ty.0.clone(), link.src_node, src_widgets_values);
        }
        wf
    }

    fn create_or_get_node(
        &mut self,
        class_type: String,
        node_id: usize,
        widgets: &[(String, NodeField)],
    ) -> &mut PromptNode {
        match self.prompt.entry(node_id.to_string()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let inputs = widgets
                    .iter()
                    .map(|(k, v)| (k.clone(), PromptInputValue::from(v.clone())))
                    .collect::<HashMap<_, _>>();

                let value = PromptNode { class_type, inputs };

                entry.insert(value)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptNode {
    #[serde(deserialize_with = "PromptInputValue::deserialize")]
    #[serde(serialize_with = "PromptInputValue::serialize")]
    pub inputs: HashMap<String, PromptInputValue>,
    pub class_type: String,
}

#[derive(Clone, Debug)]
pub enum PromptInputValue {
    Link { node_id: String, input_idx: usize },
    Text(String),
    Float(f64),
    Integer(i64),
}

impl<'de> PromptInputValue {
    fn deserialize<D>(_: D) -> Result<HashMap<String, Self>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Default::default())
    }

    fn serialize<S>(items: &HashMap<String, Self>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let items = items
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    Self::Float(value) => json!(value),
                    Self::Text(value) => json!(value),
                    Self::Integer(value) => json!(value),
                    Self::Link { node_id, input_idx } => json!([node_id, input_idx]),
                };

                (k, value)
            })
            .collect::<HashMap<_, _>>();

        items.serialize(ser)
    }
}

impl From<NodeField> for PromptInputValue {
    fn from(field: NodeField) -> PromptInputValue {
        match field {
            NodeField::IntInput(value) => Self::Integer(value.state as _),
            NodeField::FloatInput(value) => Self::Float(value.state as _),
            NodeField::StringInput(value) => Self::Text(value.state),
            NodeField::Select { state, options } => {
                let state = state
                    .and_then(|idx| options.get(idx).cloned())
                    .unwrap_or_default();

                Self::Text(state)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtraData {
    pub extra_pnginfo: ExtraPngInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtraPngInfo {
    pub workflow: Workflow,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Workflow {
    pub last_node_id: usize,
    pub last_link_id: usize,
    pub nodes: Vec<WorkflowNode>,
    #[serde(deserialize_with = "LinkItemRemote::deserialize_vec")]
    #[serde(serialize_with = "LinkItemRemote::serialize_vec")]
    pub links: Vec<LinkItem>,
    pub version: f64,
}

impl Workflow {
    pub fn new() -> Self {
        Self {
            last_node_id: 0,
            last_link_id: 0,
            nodes: vec![],
            links: vec![],
            version: 0.4,
        }
    }

    pub fn add_node(&mut self, mut node: WorkflowNode) -> usize {
        let last_link_id = self.last_link_id;

        self.last_link_id += node.inputs.len();
        self.last_node_id += 1;

        node.id = self.last_node_id;

        for (input, link_id) in node.inputs.iter_mut().zip(last_link_id..=self.last_link_id) {
            input.link = link_id;
        }

        self.nodes.push(node);

        self.last_node_id
    }

    pub fn get_node(&self, id: usize) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|x| x.id == id)
    }

    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut WorkflowNode> {
        self.nodes.iter_mut().find(|x| x.id == id)
    }

    pub fn link(&mut self, onode: usize, inode: usize, output: usize, input: usize) -> Option<()> {
        let input_node = self.get_node(inode)?;
        let input_field = input_node.inputs.get(input)?;
        let link = input_field.link;
        let ty = input_field.ty.clone();

        let link_item = LinkItem {
            link,
            out_node_id: onode,
            out_node_link_idx: output,
            in_node_id: inode,
            in_node_link_idx: input,
            ty,
        };

        self.links.push(link_item);

        let node = self.get_node_mut(onode)?;
        let output = node.outputs.get_mut(output)?;

        output.links.push(inode);

        Some(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WorkflowNode {
    pub id: usize,
    #[serde(alias = "type")]
    #[serde(rename(serialize = "type"))]
    pub ty: String,
    pub pos: (usize, usize),
    pub order: usize,
    pub mode: usize,
    #[serde(default)]
    pub inputs: Vec<WorkflowNodeInput>,
    #[serde(default)]
    pub outputs: Vec<WorkflowNodeOutput>,
    pub properties: HashMap<String, String>,
    #[serde(default)]
    pub widgets_values: Vec<Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WorkflowNodeInput {
    pub name: String,
    #[serde(alias = "type")]
    #[serde(rename(serialize = "type"))]
    pub ty: String,
    // Link ID
    pub link: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WorkflowNodeOutput {
    pub name: String,
    #[serde(alias = "type")]
    #[serde(rename(serialize = "type"))]
    pub ty: String,
    pub links: Vec<usize>,
    pub slot_index: usize,
}

#[derive(Clone, Debug)]
pub struct LinkItem {
    /// Link ID for the inputting node
    pub link: usize,
    /// Outputting node id
    pub out_node_id: usize,
    /// Outputting node link slot_index
    pub out_node_link_idx: usize,
    /// Inputting node id
    pub in_node_id: usize,
    /// Inputting node link idx
    pub in_node_link_idx: usize,
    /// Link type
    pub ty: String,
}

impl LinkItem {
    pub fn link(&self) -> usize {
        self.link
    }

    pub fn out_node_id(&self) -> usize {
        self.out_node_id
    }

    pub fn out_node_link_idx(&self) -> usize {
        self.out_node_link_idx
    }

    pub fn in_node_id(&self) -> usize {
        self.in_node_id
    }

    pub fn in_node_link_idx(&self) -> usize {
        self.in_node_link_idx
    }

    pub fn ty(&self) -> String {
        self.ty.clone()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(remote = "LinkItem")]
struct LinkItemRemote(
    #[serde(getter = "LinkItem::link")] usize,
    #[serde(getter = "LinkItem::out_node_id")] usize,
    #[serde(getter = "LinkItem::out_node_link_idx")] usize,
    #[serde(getter = "LinkItem::in_node_id")] usize,
    #[serde(getter = "LinkItem::in_node_link_idx")] usize,
    #[serde(getter = "LinkItem::ty")] String,
);

impl<'de> LinkItemRemote {
    fn serialize_vec<S>(items: &[LinkItem], ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        #[derive(Serialize)]
        struct Helper(#[serde(with = "LinkItemRemote")] LinkItem);
        let items = items.iter().map(|x| Helper(x.clone())).collect();

        Vec::serialize(&items, ser)
    }

    fn deserialize_vec<D>(de: D) -> Result<Vec<LinkItem>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "LinkItemRemote")] LinkItem);
        Ok(Vec::deserialize(de)?
            .into_iter()
            .map(|Helper(item)| item)
            .collect())
    }
}

impl From<LinkItemRemote> for LinkItem {
    fn from(raw: LinkItemRemote) -> LinkItem {
        LinkItem {
            link: raw.0,
            out_node_id: raw.1,
            out_node_link_idx: raw.2,
            in_node_id: raw.3,
            in_node_link_idx: raw.4,
            ty: raw.5,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_de_link_item() {
        let data = "[[1,2,3,4,5,\"ABC\"]]";

        let mut de = serde_json::Deserializer::from_str(data);
        let _ = LinkItemRemote::deserialize_vec(&mut de).unwrap();
    }

    #[test]
    fn test_de_workflow_simple() {
        let data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/test/test_workflow_simple.json"
        ));

        let _: Workflow = serde_json::from_str(data).unwrap();
    }

    #[test]
    fn test_de_prompt_simple() {
        const RAW: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/test/",
            "test_prompt.json"
        ));

        let _: WorkflowPrompt = serde_json::from_str(RAW).unwrap();
    }
}
