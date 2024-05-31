use indexmap::IndexMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug)]
pub struct Node {
    pub input: IndexMap<String, NodeField>,
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
        let mut inputs = IndexMap::new();

        for (k, v) in raw.input.required.into_iter() {
            let input = match v.as_slice() {
                // Cases for drop-downs and simple edge connections.
                [first] => NodeField::try_parse_selector(first)
                    .or_else(|| NodeField::try_parse_edge(first))
                    .unwrap_or(NodeField::Unknown),
                [ty, params] => {
                    NodeField::try_parse_input(ty, params).unwrap_or(NodeField::Unknown)
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

// TODO: Turn into a flat enum later.
pub const TY_INT: &str = "Kira__Reserved_Int";
pub const TY_FLOAT: &str = "Kira__Reserved_Float";
pub const TY_STRING: &str = "Kira__Reserved_String";
pub const TY_SELECT: &str = "Kira__Reserved_Select";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeField {
    IntInput(IntInput),
    FloatInput(FloatInput),
    StringInput(StringInput),
    BoolInput(BoolInput),
    Select {
        options: Vec<String>,
        state: Option<usize>,
    },
    Connection(String),
    Unknown,
}

impl NodeField {
    pub fn ty(&self) -> String {
        use NodeField::*;

        match self {
            IntInput(_) => TY_INT.into(),
            FloatInput(_) => TY_FLOAT.into(),
            StringInput(_) => TY_STRING.into(),
            BoolInput(_) => "bool".into(),
            Select { .. } => TY_SELECT.into(),
            Connection(ty) => ty.into(),
            Unknown => "unknown".into(),
        }
    }

    pub fn is_connection(&self) -> bool {
        matches!(self, NodeField::Connection(_))
    }

    pub fn options(&self) -> Option<Vec<String>> {
        match self {
            Self::Select { options, .. } => Some(options.clone()),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<String> {
        match self {
            Self::StringInput(input) => Some(input.state.clone()),
            Self::Select { options, state } => state.and_then(|s| options.get(s)).cloned(),
            _ => None,
        }
    }

    pub fn float(&self) -> Option<f32> {
        match self {
            Self::FloatInput(input) => Some(input.state),
            Self::IntInput(input) => Some(input.state),
            _ => None,
        }
    }

    pub fn option(&self) -> Option<usize> {
        if let Self::Select { state, .. } = self {
            return *state;
        }

        None
    }

    pub fn set_text(&mut self, text: String) {
        match self {
            Self::StringInput(ref mut input) => {
                input.state = text;
            }
            Self::Select {
                ref options,
                ref mut state,
            } => {
                *state = options.iter().position(|x| x == text.as_str());
            }
            _ => (),
        }
    }

    pub fn set_int(&mut self, value: usize) {
        if let Self::IntInput(ref mut input) = self {
            input.state = value as _;
        }
    }

    pub fn set_float(&mut self, value: f32) {
        if let Self::FloatInput(ref mut input) = self {
            input.state = value;
        }
    }

    fn try_parse_selector(first: &serde_json::Value) -> Option<Self> {
        // TODO: VHS_VideoCombine and nested drop downs are currently not supported.
        let options = first
            .as_array()?
            .iter()
            .filter_map(|choice| choice.as_str())
            .map(ToOwned::to_owned)
            .collect();

        Some(Self::Select {
            options,
            state: None,
        })
    }

    fn try_parse_edge(first: &serde_json::Value) -> Option<Self> {
        first.as_str().map(ToOwned::to_owned).map(Self::Connection)
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

    pub fn state(&self) -> Option<Value> {
        use NodeField::*;

        let value = match self {
            IntInput(input) => json!(input.state as i32),
            FloatInput(input) => json!(input.state),
            StringInput(input) => json!(input.state),
            BoolInput(input) => json!(input.default),
            Select { state, .. } => json!(state),
            Connection(_) => return None,
            Unknown => return None,
        };

        Some(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntInput {
    pub default: f32,
    pub min: f32,
    #[serde(default = "f32_max")]
    pub max: f32,
    #[serde(default = "float_step")]
    pub step: f32,
    #[serde(default = "float_step")]
    pub state: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloatInput {
    pub default: f32,
    pub min: f32,
    #[serde(default = "f32_max")]
    pub max: f32,
    #[serde(default = "float_step")]
    pub step: f32,
    #[serde(default = "float_step")]
    pub state: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StringInput {
    #[serde(default = "bool_false")]
    pub multiline: bool,
    pub default: Option<String>,
    #[serde(default)]
    pub state: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoolInput {
    pub default: bool,
}

#[derive(Deserialize, Debug)]
pub(super) struct RawNode {
    input: RawInput,
    output: Vec<String>,
    #[allow(dead_code)]
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
    required: IndexMap<String, Vec<Value>>,
    // Nodes with hidden inputs:
    // - SaveImage
    // - PreviewImage
    #[allow(dead_code)]
    hidden: Option<Value>,
    // nodes with optional inputs:
    // - LatentCompositeMasked
    #[allow(dead_code)]
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
