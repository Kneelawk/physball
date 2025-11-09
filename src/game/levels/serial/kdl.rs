use crate::game::levels::serial::level::{SerialLevel, SerialPlane, SerialText};
use bevy::asset::LoadContext;
use bevy::prelude::*;
use bevy_rich_text3d::TextAlign;
use std::f32::consts::PI;

#[derive(Debug, Clone, knus::Decode)]
pub struct KdlLevel {
    #[knus(child)]
    spawn: KdlLocation,

    #[knus(child)]
    finish: KdlLocation,

    #[knus(children(name = "plane"))]
    planes: Vec<KdlPlane>,

    #[knus(children(name = "text"))]
    texts: Vec<KdlText>,
    // TODO: death planes
}

#[derive(Debug, Clone, knus::Decode)]
pub struct KdlLocation {
    #[knus(child)]
    pos: KdlVec3,

    #[knus(children(name = "rot"))]
    rotations: Vec<KdlRotation>,
}

#[derive(Debug, Clone, knus::Decode)]
pub struct KdlPlane {
    #[knus(child)]
    pos: KdlVec3,

    #[knus(children(name = "rot"))]
    rotations: Vec<KdlRotation>,

    #[knus(argument)]
    size: f32,

    #[knus(argument)]
    size2: Option<f32>,

    #[knus(property(name = "type"), default)]
    ty: KdlPlaneType,
}

impl KdlPlane {
    pub fn into_serial_plane(self) -> SerialPlane {
        SerialPlane {
            width: self.size,
            length: self.size2.unwrap_or(self.size),
            trans: Transform::from_rotation(self.rotations.into_quat())
                .with_translation(self.pos.into_vec()),
            ty: self.ty,
        }
    }
}

#[derive(
    Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, knus::DecodeScalar, Reflect,
)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub enum KdlPlaneType {
    #[default]
    Static,
    Death,
}

#[derive(Debug, Clone, knus::Decode)]
pub struct KdlText {
    #[knus(argument)]
    text: String,

    #[knus(property, default = 64.0)]
    pt: f32,

    #[knus(property)]
    font: Option<String>,

    #[knus(property, default)]
    align: KdlAlign,

    #[knus(child)]
    pos: KdlVec3,

    #[knus(children(name = "rot"))]
    rotations: Vec<KdlRotation>,

    #[knus(child)]
    scale: Option<KdlScale>,
}

impl KdlText {
    pub fn into_serial_text(self) -> SerialText {
        SerialText {
            text: self.text,
            pt: self.pt,
            font: self.font,
            align: self.align,
            trans: Transform::from_scale(self.scale.map(KdlScale::into_vec).unwrap_or(Vec3::ONE))
                .with_rotation(self.rotations.into_quat())
                .with_translation(self.pos.into_vec()),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, knus::DecodeScalar, Reflect)]
#[reflect(Debug, Default, Clone)]
pub enum KdlAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl KdlAlign {
    pub fn to_text_align(self) -> TextAlign {
        match self {
            KdlAlign::Left => TextAlign::Left,
            KdlAlign::Center => TextAlign::Center,
            KdlAlign::Right => TextAlign::Right,
        }
    }
}

#[derive(Debug, Copy, Clone, knus::Decode)]
pub struct KdlScale {
    #[knus(argument)]
    scale1: f32,
    #[knus(argument)]
    scale2: Option<f32>,
    #[knus(argument)]
    scale3: Option<f32>,
}

impl KdlScale {
    pub fn into_vec(self) -> Vec3 {
        Vec3::new(
            self.scale1,
            self.scale2.unwrap_or(self.scale1),
            self.scale3.or(self.scale2).unwrap_or(self.scale1),
        )
    }
}

#[derive(Debug, Copy, Clone, knus::Decode)]
pub struct KdlRotation {
    #[knus(argument)]
    axis: KdlAxis,
    #[knus(argument)]
    angle: f32,
}

#[derive(Debug, Copy, Clone, knus::DecodeScalar)]
pub enum KdlAxis {
    X,
    Y,
    Z,
}

pub trait IntoQuat {
    fn into_quat(self) -> Quat;
}

impl IntoQuat for KdlRotation {
    fn into_quat(self) -> Quat {
        let axis = match self.axis {
            KdlAxis::X => Vec3::X,
            KdlAxis::Y => Vec3::Y,
            KdlAxis::Z => Vec3::Z,
        };

        Quat::from_axis_angle(axis, self.angle / 180.0 * PI)
    }
}

impl IntoQuat for Vec<KdlRotation> {
    fn into_quat(self) -> Quat {
        self.into_iter()
            .fold(Quat::IDENTITY, |q, a| a.into_quat() * q)
    }
}

#[derive(Debug, Copy, Clone, knus::Decode)]
pub struct KdlVec3 {
    #[knus(argument)]
    x: f32,
    #[knus(argument)]
    y: f32,
    #[knus(argument)]
    z: f32,
}

impl KdlVec3 {
    pub fn into_vec(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl KdlLevel {
    pub fn bind(self, _load_context: &mut LoadContext<'_>) -> SerialLevel {
        SerialLevel {
            spawn: Transform::from_rotation(self.spawn.rotations.into_quat())
                .with_translation(self.spawn.pos.into_vec()),
            finish: Transform::from_rotation(self.finish.rotations.into_quat())
                .with_translation(self.finish.pos.into_vec()),
            planes: self
                .planes
                .into_iter()
                .map(KdlPlane::into_serial_plane)
                .collect(),
            texts: self
                .texts
                .into_iter()
                .map(KdlText::into_serial_text)
                .collect(),
        }
    }
}
