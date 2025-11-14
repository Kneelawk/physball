use crate::game::levels::serial::BindArgs;
use crate::game::levels::serial::error::{BindArgsErrorExt, KdlBindError};
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue, NodeKey};
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum KdlValueType {
    String,
    Integer,
    Float,
    Bool,
    Null,
}

pub trait DisplayValueType {
    fn display_type(&self) -> String;
}

impl Display for KdlValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KdlValueType::String => write!(f, "string"),
            KdlValueType::Integer => write!(f, "integer"),
            KdlValueType::Float => write!(f, "float"),
            KdlValueType::Bool => write!(f, "bool"),
            KdlValueType::Null => write!(f, "null"),
        }
    }
}

impl DisplayValueType for KdlValueType {
    fn display_type(&self) -> String {
        match self {
            KdlValueType::String => "string",
            KdlValueType::Integer => "integer",
            KdlValueType::Float => "float",
            KdlValueType::Bool => "bool",
            KdlValueType::Null => "null",
        }
        .to_string()
    }
}

impl DisplayValueType for &[KdlValueType] {
    fn display_type(&self) -> String {
        match self.len() {
            2 => {
                format!("{} and {}", &self[0], &self[1])
            }
            _ => {
                let mut str = String::new();
                for i in 0..self.len() {
                    if i > 0 {
                        str += ", ";
                    }
                    if i >= self.len() - 1 {
                        str += "or ";
                    }

                    str += &self[i].display_type();
                }
                str
            }
        }
    }
}

pub trait KdlDocumentExt {
    fn must_get(&self, name: &str, args: &BindArgs) -> Result<&KdlNode, KdlBindError>;
}

impl KdlDocumentExt for KdlDocument {
    fn must_get(&self, name: &str, args: &BindArgs) -> Result<&KdlNode, KdlBindError> {
        self.get(name).ok_or(args.missing_element(name))
    }
}

pub trait KdlNodeExt {
    fn must_children(&self, args: &BindArgs) -> Result<&KdlDocument, KdlBindError>;

    fn must_entry(
        &self,
        key: impl Into<NodeKey>,
        args: &BindArgs,
    ) -> Result<&KdlEntry, KdlBindError>;

    fn must_get(&self, key: impl Into<NodeKey>, args: &BindArgs)
    -> Result<&KdlValue, KdlBindError>;

    fn must_get_number(
        &self,
        key: impl Into<NodeKey>,
        args: &BindArgs,
    ) -> Result<f64, KdlBindError>;

    fn must_get_string(
        &self,
        key: impl Into<NodeKey>,
        args: &BindArgs,
    ) -> Result<&str, KdlBindError>;
}

impl KdlNodeExt for KdlNode {
    fn must_children(&self, args: &BindArgs) -> Result<&KdlDocument, KdlBindError> {
        self.children()
            .ok_or(args.no_children(self.span(), self.name()))
    }

    fn must_entry(
        &self,
        key: impl Into<NodeKey>,
        args: &BindArgs,
    ) -> Result<&KdlEntry, KdlBindError> {
        let key = key.into();
        self.entry(key.clone())
            .ok_or(args.no_entry(key, self.span(), self.name()))
    }

    fn must_get(
        &self,
        key: impl Into<NodeKey>,
        args: &BindArgs,
    ) -> Result<&KdlValue, KdlBindError> {
        let key = key.into();
        self.get(key.clone())
            .ok_or(args.no_entry(key, self.span(), self.name()))
    }

    fn must_get_number(
        &self,
        key: impl Into<NodeKey>,
        args: &BindArgs,
    ) -> Result<f64, KdlBindError> {
        let key = key.into();
        let entry = self.must_entry(key.clone(), args)?;
        match entry.value() {
            KdlValue::String(_) => {}
            KdlValue::Integer(val) => return Ok(*val as f64),
            KdlValue::Float(val) => return Ok(*val),
            KdlValue::Bool(_) => {}
            KdlValue::Null => {}
        }

        Err(args.wrong_value_type(
            key,
            entry.value().value_type(),
            &[KdlValueType::Integer, KdlValueType::Float],
            entry.span(),
            self.name(),
        ))
    }

    fn must_get_string(&self, key: impl Into<NodeKey>, args: &BindArgs) -> Result<&str, KdlBindError> {
        let key = key.into();
        let entry = self.must_entry(key.clone(), args)?;
        match entry.value() {
            KdlValue::String(_) => {}
            KdlValue::Integer(_) => {},
            KdlValue::Float(_) => {},
            KdlValue::Bool(_) => {}
            KdlValue::Null => {}
        }

        Err(args.wrong_value_type(
            key,
            entry.value().value_type(),
            &[KdlValueType::Integer, KdlValueType::Float],
            entry.span(),
            self.name(),
        ))
    }
}

pub trait KdlValueExt {
    fn value_type(&self) -> KdlValueType;

    fn as_number(&self) -> Option<f64>;

    fn as_string(&self) -> Option<&str>;
}

impl KdlValueExt for KdlValue {
    fn value_type(&self) -> KdlValueType {
        match self {
            KdlValue::String(_) => KdlValueType::String,
            KdlValue::Integer(_) => KdlValueType::Integer,
            KdlValue::Float(_) => KdlValueType::Float,
            KdlValue::Bool(_) => KdlValueType::Bool,
            KdlValue::Null => KdlValueType::Null,
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            KdlValue::String(_) => None,
            KdlValue::Integer(val) => Some(*val as f64),
            KdlValue::Float(val) => Some(*val),
            KdlValue::Bool(_) => None,
            KdlValue::Null => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            KdlValue::String(val) => Some(val.as_ref()),
            KdlValue::Integer(_) => None,
            KdlValue::Float(_) => None,
            KdlValue::Bool(_) => None,
            KdlValue::Null => None,
        }
    }
}
