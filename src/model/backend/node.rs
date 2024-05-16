use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug)]
pub struct Node {
    pub input: HashMap<String, NodeField>,
    pub output: Vec<String>,
    pub output_name: Vec<String>,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub output_node: bool,
}

impl From<RawNode> for Node {
    fn from(raw: RawNode) -> Self {
        let mut inputs = HashMap::new();

        for (k, v) in raw.input.required.into_iter() {
            println!("K: {k}");

            let input = match v.as_slice() {
                // Cases for drop-downs and simple edge connections.
                [first] => {
                    NodeField::try_parse_selector(&first)
                        .or_else(|| NodeField::try_parse_edge(&first))
                        .unwrap_or(NodeField::Unknown)
                }
                [ty, params] => {
                    NodeField::try_parse_input(ty, params)
                        .unwrap_or(NodeField::Unknown)
                }
                _ => {
                    panic!("Exepcted two items in node field");
                }
            };

            inputs.insert(k, input);
        }

        Node { 
            input: inputs,
            output: raw.output,
            output_name: raw.output_name,
            name: raw.name,
            display_name: raw.display_name,
            description: raw.description,
            category: raw.category,
            output_node: raw.output_node,
        }
    }
}
#[derive(Debug)]
pub enum NodeField {
    IntInput(IntInput),
    FloatInput(FloatInput),
    StringInput(StringInput),
    BoolInput(BoolInput),
    Select(Vec<String>),
    Connection(String),
    Unknown
}

impl NodeField {
    pub fn ty(&self) -> String {
        use NodeField::*;

        match self {
            IntInput(_) => "int".into(),
            FloatInput(_) => "float".into(),
            StringInput(_) => "string".into(),
            BoolInput(_) => "bool".into(),
            Select(_) => "select".into(),
            Connection(ty) => ty.into(),
            Unknown => "unknown".into(),
        }
    }

    fn try_parse_selector(first: &serde_json::Value) -> Option<Self> {
        // TODO: VHS_VideoCombine and nested drop downs are currently not supported.
        let items = first
            .as_array()?
            .iter()
            .filter_map(|choice| choice.as_str())
            .map(ToOwned::to_owned)
            .collect();

        Some(Self::Select(items))
    }

    fn try_parse_edge(first: &serde_json::Value) -> Option<Self> {
        first
            .as_str()
            .map(ToOwned::to_owned)
            .map(Self::Connection)
    }

    fn try_parse_input(ty: &serde_json::Value, second: &serde_json::Value) -> Option<Self> {
        let ty = ty.as_str()?;

        fn try_parse<T: serde::de::DeserializeOwned>(second: &serde_json::Value) -> T {
            serde_path_to_error::deserialize(second).expect("Failed to deserialize input.")
        }

        let item = match ty {
            "FLOAT" => Self::FloatInput(try_parse(second)),
            "INT" => Self::IntInput(try_parse(second)),
            "STRING" => Self::StringInput(try_parse(second)),
            "BOOLEAN" => Self::BoolInput(try_parse(second)),
            unk => panic!("Tried to parse unknown input type: {}", unk),
        };

        Some(item)
    }
}

#[derive(Deserialize, Debug)]
pub struct IntInput {
    pub default: f32,
    pub min: f32,
    #[serde(default = "f32_max")]
    pub max: f32,
    #[serde(default = "float_step")]
    pub step: f32,
}

#[derive(Deserialize, Debug)]
pub struct FloatInput {
    pub default: f32,
    pub min: f32,
    #[serde(default = "f32_max")]
    pub max: f32,
    #[serde(default = "float_step")]
    pub step: f32,
}

#[derive(Deserialize, Debug)]
pub struct StringInput {
    #[serde(default = "bool_false")]
    pub multiline: bool,
    pub default: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BoolInput {
    pub default: bool,
}

#[derive(Deserialize, Debug)]
pub(super) struct RawNode {
    input: RawInput,
    output: Vec<String>,
    output_is_list: Vec<bool>,
    output_name: Vec<String>,
    name: String,
    display_name: String,
    description: String,
    category: String,
    output_node: bool,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct RawInput {
    required: HashMap<String, Vec<Value>>,
    // Nodes with hidden inputs:
    // - SaveImage
    // - PreviewImage
    hidden: Option<Value>,
    // nodes with optional inputs:
    // - LatentCompositeMasked
    optional: Option<Value>,
}

fn float_step() -> f32 {
    0.01
}

fn bool_false() -> bool {
    false
}

fn f32_max() -> f32 {
    f32::MAX
}
