#![allow(dead_code)]

use crate::game::assets::{AssetType, asset_ref};
use crate::game::levels::serial::error::{BindErrorExt, KdlBindError, MergeKdlBindError};
use bevy::asset::LoadContext;
use bevy::prelude::*;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue, NodeKey};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use strum::VariantArray;

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
            1 => self[0].to_string(),
            2 => {
                format!("either {} or {}", &self[0], &self[1])
            }
            _ => {
                let mut str = "one of ".to_string();
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
    fn must_get(&self, name: &str, args: &Arc<String>) -> Result<&KdlNode, KdlBindError>;

    fn get(&self, name: &str) -> Option<&KdlNode>;

    fn nodes(&self) -> &[KdlNode];

    fn children(&self, name: &str) -> Option<&KdlDocument>;

    fn must_children(&self, name: &str, args: &Arc<String>) -> Result<&KdlDocument, KdlBindError>;

    fn get_transform(&self, args: &Arc<String>) -> Result<Transform, KdlBindError> {
        let mut trans = Transform::default();

        let translation = self
            .get("pos")
            .map_or(Ok(None), |node| node.must_get_vec3(0, args).map(Some));
        let scale = self
            .get("scale")
            .map_or(Ok(None), |node| node.must_get_scale(0, args).map(Some));
        let rotations = self
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "rot")
            .map(|node| node.must_get_rotation(0, args))
            .collect::<Vec<_>>()
            .merge();

        let (translation, scale, rotations) = (translation, scale, rotations).merge()?;

        if let Some(translation) = translation {
            trans.translation = translation;
        }
        if let Some(scale) = scale {
            trans.scale = scale;
        }
        trans.rotation = rotations.iter().fold(Quat::IDENTITY, |q, a| a * q);

        Ok(trans)
    }
}

impl KdlDocumentExt for KdlDocument {
    fn must_get(&self, name: &str, args: &Arc<String>) -> Result<&KdlNode, KdlBindError> {
        self.get(name).ok_or(args.missing_element(name))
    }

    fn get(&self, name: &str) -> Option<&KdlNode> {
        self.get(name)
    }

    fn nodes(&self) -> &[KdlNode] {
        self.nodes()
    }

    fn children(&self, name: &str) -> Option<&KdlDocument> {
        self.get(name).and_then(|n| n.children())
    }

    fn must_children(&self, name: &str, args: &Arc<String>) -> Result<&KdlDocument, KdlBindError> {
        self.must_get(name, args)?.must_children(args)
    }
}

pub trait KdlNodeExt {
    fn must_children(&self, args: &Arc<String>) -> Result<&KdlDocument, KdlBindError>;

    fn must_child(&self, name: &str, source: &Arc<String>) -> Result<&KdlNode, KdlBindError>;

    fn child(&self, name: &str) -> Option<&KdlNode>;

    fn must_entry(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<&KdlEntry, KdlBindError>;

    fn entry(&self, key: impl Into<NodeKey>) -> Option<&KdlEntry>;

    fn must_get(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<&KdlValue, KdlBindError>;

    fn get(&self, key: impl Into<NodeKey>) -> Option<&KdlValue>;

    fn must_get_number(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<f64, KdlBindError>;

    fn get_number(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<Option<f64>, KdlBindError>;

    fn must_get_string(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<&str, KdlBindError>;

    fn get_string(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<Option<&str>, KdlBindError>;

    fn must_get_parse<T: FromStr>(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<T, KdlBindError>
    where
        T::Err: Display;

    fn get_parse<T: FromStr>(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<Option<T>, KdlBindError>
    where
        T::Err: Display;

    fn must_get_handle<A: Asset + AssetType>(
        &self,
        key: impl Into<NodeKey>,
        load_context: &mut LoadContext,
        source: &Arc<String>,
    ) -> Result<Handle<A>, KdlBindError>;

    fn get_handle<A: Asset + AssetType>(
        &self,
        key: impl Into<NodeKey>,
        load_context: &mut LoadContext,
        source: &Arc<String>,
    ) -> Result<Option<Handle<A>>, KdlBindError>;

    fn must_get_variant<'t, T: Display>(
        &self,
        key: impl Into<NodeKey>,
        variants: &'t [T],
        source: &Arc<String>,
    ) -> Result<&'t T, KdlBindError>;

    fn get_variant<'t, T: Display>(
        &self,
        key: impl Into<NodeKey>,
        variants: &'t [T],
        source: &Arc<String>,
    ) -> Result<Option<&'t T>, KdlBindError>;

    fn must_get_vec3(&self, arg_offset: usize, source: &Arc<String>) -> Result<Vec3, KdlBindError> {
        let x = self.must_get_number(arg_offset, source);
        let y = self.must_get_number(arg_offset + 1, source);
        let z = self.must_get_number(arg_offset + 2, source);
        let (x, y, z) = (x, y, z).merge()?;
        Ok(Vec3::new(x as f32, y as f32, z as f32))
    }

    fn must_get_scale(
        &self,
        arg_offset: usize,
        source: &Arc<String>,
    ) -> Result<Vec3, KdlBindError> {
        let x = self.must_get_number(arg_offset, source)?;
        let y = self.get(arg_offset + 1).and_then(KdlValue::as_number);
        let z = self.get(arg_offset + 2).and_then(KdlValue::as_number);
        Ok(Vec3::new(
            x as f32,
            y.unwrap_or(x) as f32,
            z.unwrap_or(x) as f32,
        ))
    }

    fn must_get_rotation(
        &self,
        arg_offset: usize,
        source: &Arc<String>,
    ) -> Result<Quat, KdlBindError> {
        let axis = self
            .must_get_variant(arg_offset, KdlAxis::VARIANTS, source)
            .map(|axis| axis.to_vec3());
        let rotation = self.must_get_number(arg_offset + 1, source);
        let (axis, rotation) = (axis, rotation).merge()?;
        Ok(Quat::from_axis_angle(
            axis,
            (rotation * std::f64::consts::PI / 180.0) as f32,
        ))
    }
}

impl KdlNodeExt for KdlNode {
    fn must_children(&self, source: &Arc<String>) -> Result<&KdlDocument, KdlBindError> {
        self.children()
            .ok_or_else(|| source.no_children(self.span()))
    }

    fn must_child(&self, name: &str, source: &Arc<String>) -> Result<&KdlNode, KdlBindError> {
        self.must_children(source)
            .and_then(|children| children.must_get(name, source))
    }

    fn child(&self, name: &str) -> Option<&KdlNode> {
        self.children().and_then(|children| children.get(name))
    }

    fn must_entry(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<&KdlEntry, KdlBindError> {
        let key = key.into();
        self.entry(key.clone())
            .ok_or_else(|| source.no_entry(key, self.span()))
    }

    fn entry(&self, key: impl Into<NodeKey>) -> Option<&KdlEntry> {
        self.entry(key)
    }

    fn must_get(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<&KdlValue, KdlBindError> {
        let key = key.into();
        self.get(key.clone())
            .ok_or_else(|| source.no_entry(key, self.span()))
    }

    fn get(&self, key: impl Into<NodeKey>) -> Option<&KdlValue> {
        self.get(key)
    }

    fn must_get_number(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<f64, KdlBindError> {
        self.must_entry(key, source)?.as_number(source)
    }

    fn get_number(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<Option<f64>, KdlBindError> {
        self.entry(key)
            .map_or(Ok(None), |e| e.as_number(source).map(Some))
    }

    fn must_get_string(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<&str, KdlBindError> {
        self.must_entry(key, source)?.as_string(source)
    }

    fn get_string(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<Option<&str>, KdlBindError> {
        self.entry(key)
            .map_or(Ok(None), |e| e.as_string(source).map(Some))
    }

    fn must_get_parse<T: FromStr>(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<T, KdlBindError>
    where
        T::Err: Display,
    {
        self.must_entry(key, source)?.as_parse(source)
    }

    fn get_parse<T: FromStr>(
        &self,
        key: impl Into<NodeKey>,
        source: &Arc<String>,
    ) -> Result<Option<T>, KdlBindError>
    where
        T::Err: Display,
    {
        self.entry(key)
            .map_or(Ok(None), |e| e.as_parse(source).map(Some))
    }

    fn must_get_handle<A: Asset + AssetType>(
        &self,
        key: impl Into<NodeKey>,
        load_context: &mut LoadContext,
        source: &Arc<String>,
    ) -> Result<Handle<A>, KdlBindError> {
        self.must_entry(key, source)?
            .as_handle(load_context, source)
    }

    fn get_handle<A: Asset + AssetType>(
        &self,
        key: impl Into<NodeKey>,
        load_context: &mut LoadContext,
        source: &Arc<String>,
    ) -> Result<Option<Handle<A>>, KdlBindError> {
        self.entry(key)
            .map_or(Ok(None), |e| e.as_handle(load_context, source).map(Some))
    }

    fn must_get_variant<'t, T: Display>(
        &self,
        key: impl Into<NodeKey>,
        variants: &'t [T],
        source: &Arc<String>,
    ) -> Result<&'t T, KdlBindError> {
        self.must_entry(key, source)?.as_variant(variants, source)
    }

    fn get_variant<'t, T: Display>(
        &self,
        key: impl Into<NodeKey>,
        variants: &'t [T],
        source: &Arc<String>,
    ) -> Result<Option<&'t T>, KdlBindError> {
        self.entry(key)
            .map_or(Ok(None), |e| e.as_variant(variants, source).map(Some))
    }
}

pub trait KdlEntryExt {
    fn value_type(&self) -> KdlValueType;

    fn as_number(&self, source: &Arc<String>) -> Result<f64, KdlBindError>;

    fn as_string(&self, source: &Arc<String>) -> Result<&str, KdlBindError>;

    fn as_parse<T: FromStr>(&self, source: &Arc<String>) -> Result<T, KdlBindError>
    where
        T::Err: Display;

    fn as_handle<A: Asset + AssetType>(
        &self,
        load_context: &mut LoadContext,
        source: &Arc<String>,
    ) -> Result<Handle<A>, KdlBindError>;

    fn as_variant<'t, T: Display>(
        &self,
        variants: &'t [T],
        source: &Arc<String>,
    ) -> Result<&'t T, KdlBindError>;
}

impl KdlEntryExt for KdlEntry {
    fn value_type(&self) -> KdlValueType {
        self.value().value_type()
    }

    fn as_number(&self, source: &Arc<String>) -> Result<f64, KdlBindError> {
        self.value().as_number().ok_or_else(|| {
            source.wrong_value_type(
                self.value().value_type(),
                &[KdlValueType::Integer, KdlValueType::Float],
                self.span(),
            )
        })
    }

    fn as_string(&self, source: &Arc<String>) -> Result<&str, KdlBindError> {
        self.value().as_string().ok_or_else(|| {
            source.wrong_value_type(
                self.value().value_type(),
                &[KdlValueType::String],
                self.span(),
            )
        })
    }

    fn as_parse<T: FromStr>(&self, source: &Arc<String>) -> Result<T, KdlBindError>
    where
        T::Err: Display,
    {
        self.as_string(source)?
            .parse()
            .map_err(|err| source.parse_error(err, self.span()))
    }

    fn as_handle<A: Asset + AssetType>(
        &self,
        load_context: &mut LoadContext,
        source: &Arc<String>,
    ) -> Result<Handle<A>, KdlBindError> {
        asset_ref::load(self.as_string(source)?, load_context)
            .map_err(|err| source.parse_error(err, self.span()))
    }

    fn as_variant<'t, T: Display>(
        &self,
        variants: &'t [T],
        source: &Arc<String>,
    ) -> Result<&'t T, KdlBindError> {
        let str = self.as_string(source)?;
        for var in variants {
            if var.to_string() == str {
                return Ok(var);
            }
        }
        Err(source.not_a_variant(str, variants, self.span()))
    }
}

pub trait KdlValueExt {
    fn value_type(&self) -> KdlValueType;

    fn as_number(&self) -> Option<f64>;
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
}

#[derive(Debug, Copy, Clone, strum::VariantArray, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum KdlAxis {
    X,
    Y,
    Z,
}

impl KdlAxis {
    pub fn to_vec3(self) -> Vec3 {
        match self {
            KdlAxis::X => Vec3::X,
            KdlAxis::Y => Vec3::Y,
            KdlAxis::Z => Vec3::Z,
        }
    }
}
