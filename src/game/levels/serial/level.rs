use crate::capture_result;
use crate::game::assets::asset_ref;
use crate::game::assets::asset_ref::default_plane_material;
use crate::game::assets::fonts::FontNames;
use crate::game::assets::preload::Preloads;
use crate::game::levels::button::LevelButton;
use crate::game::levels::death::DeathCollider;
use crate::game::levels::finish_point::FinishPoint;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::{LevelObject, PlayerSpawnPoint};
use avian3d::prelude::*;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAlign};
use kdl::{KdlDocument, KdlNode};
use std::collections::HashMap;
use std::string::ToString;
use std::sync::Arc;
use strum::VariantArray;

pub const DEFAULT_TEXT_PT: f64 = 64.0;

pub struct LevelBuildArgs<'a, 'w, 's> {
    /// Whether to spawn the assets that would otherwise be spawned by a checkpoint load
    pub dyn_assets: bool,
    pub cmd: &'a mut Commands<'w, 's>,
    pub assets: &'a AssetServer,
    pub preloads: &'a Preloads,
    pub fonts: &'a FontNames,
}

#[derive(Debug, Clone, Asset, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialLevel {
    pub spawn: Transform,
    pub finish: Transform,
    pub planes: Vec<SerialPlane>,
    pub texts: Vec<SerialText>,
    pub buttons: Vec<SerialButton>,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialPlane {
    pub width: f32,
    pub length: f32,
    pub trans: Transform,
    pub material: Handle<StandardMaterial>,
    pub ty: SerialPlaneType,
}

#[derive(Debug, Default, Copy, Clone, Reflect, strum::VariantArray, strum::Display)]
#[reflect(Debug, Default, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum SerialPlaneType {
    #[default]
    Static,
    Death,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialText {
    pub text: String,
    pub trans: Transform,
    pub material: Handle<StandardMaterial>,
    pub pt: f32,
    pub font: Handle<Font>,
    pub align: SerialAlign,
}

#[derive(Debug, Default, Copy, Clone, Reflect, strum::VariantArray, strum::Display)]
#[reflect(Debug, Default, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum SerialAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialButton {
    pub name: String,
    pub trans: Transform,
}

pub struct SerialButtonDoor {}

impl SerialLevel {
    pub fn bind(
        doc: &KdlDocument,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let spawn = capture_result! {
            let doc = doc.must_children("spawn", &source)?;
            doc.get_transform(&source)
        };

        let finish = capture_result! {
            let doc = doc.must_children("finish", &source)?;
            doc.get_transform(&source)
        };

        let planes = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "plane")
            .map(|node| SerialPlane::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let texts = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "text")
            .map(|node| SerialText::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let buttons = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "button")
            .map(|node| SerialButton::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let (spawn, finish, planes, texts, buttons) =
            (spawn, finish, planes, texts, buttons).merge()?;

        Ok(Self {
            spawn,
            finish,
            planes,
            texts,
            buttons,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((LevelObject, PlayerSpawnPoint, self.spawn));
        args.cmd.spawn((LevelObject, FinishPoint, self.finish));

        for plane in self.planes.iter() {
            plane.spawn(args);
        }

        for text in self.texts.iter() {
            text.spawn(args);
        }

        let mut button_names = HashMap::new();
        for button in self.buttons.iter() {
            button_names.insert(button.name.clone(), button.spawn(args));
        }
    }
}

impl SerialPlane {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let size = node.must_get_number(0, &source);
        let size2 = node.get_number(1, &source);

        let ty = node
            .get_variant("type", SerialPlaneType::VARIANTS, &source)
            .map(|ty| ty.copied().unwrap_or_default());

        let material = node
            .get_handle("material", load_context, &source)
            .map(|handle| handle.unwrap_or_else(|| default_plane_material(load_context)));

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (size, size2, ty, material, trans) = (size, size2, ty, material, trans).merge()?;

        Ok(SerialPlane {
            width: size as f32,
            length: size2.unwrap_or(size) as f32,
            trans,
            material,
            ty,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        match self.ty {
            SerialPlaneType::Static => {
                args.cmd.spawn((
                    LevelObject,
                    self.trans,
                    Mesh3d(
                        args.assets.add(
                            Plane3d::new(Vec3::Y, Vec2::new(self.width / 2.0, self.length / 2.0))
                                .into(),
                        ),
                    ),
                    MeshMaterial3d(self.material.clone()),
                    children![(
                        RigidBody::Static,
                        Collider::cuboid(self.width, 0.2, self.length),
                        Transform::from_xyz(0.0, -0.1, 0.0)
                    )],
                ));
            }
            SerialPlaneType::Death => {
                args.cmd.spawn((
                    LevelObject,
                    self.trans
                        .with_translation(self.trans.translation + vec3(0.0, -0.1, 0.0)),
                    RigidBody::Static,
                    Collider::cuboid(self.width, 0.2, self.length),
                    DeathCollider,
                ));
            }
        }
    }
}

impl SerialText {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let text = node
            .must_get_string(0, &source)
            .map(|text| text.to_string());

        let pt = node
            .get_number("pt", &source)
            .map(|pt| pt.unwrap_or(DEFAULT_TEXT_PT));

        let font = node
            .get_handle::<Font>("font", load_context, &source)
            .map(|asset| asset.unwrap_or_else(|| asset_ref::default_font(load_context)));

        let align = node
            .get_variant("align", SerialAlign::VARIANTS, &source)
            .map(|align| align.copied().unwrap_or_default());

        let material = node
            .get_handle("material", load_context, &source)
            .map(|asset| asset.unwrap_or_else(|| asset_ref::default_text_material(load_context)));

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (text, pt, font, align, material, trans) =
            (text, pt, font, align, material, trans).merge()?;

        Ok(Self {
            text,
            trans,
            material,
            pt: pt as f32,
            font,
            align,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        let font = args
            .fonts
            .get(&self.font.id())
            .unwrap_or_else(|| {
                warn!("Font {:?} has not loaded yet, using default", &self.font);
                &args.fonts[&args.preloads.text_font().id()]
            })
            .clone();
        args.cmd.spawn((
            LevelObject,
            self.trans,
            Text3d::new(self.text.clone()),
            Text3dStyling {
                font: font.into(),
                size: self.pt,
                world_scale: Some(Vec2::splat(self.pt / 256.0)),
                align: self.align.to_text_align(),
                ..Default::default()
            },
            Mesh3d::default(),
            MeshMaterial3d(self.material.clone()),
        ));
    }
}

impl SerialAlign {
    pub fn to_text_align(self) -> TextAlign {
        match self {
            SerialAlign::Left => TextAlign::Left,
            SerialAlign::Center => TextAlign::Center,
            SerialAlign::Right => TextAlign::Right,
        }
    }
}

impl SerialButton {
    pub fn bind(
        node: &KdlNode,
        _load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let name = node
            .must_get_string(0, &source)
            .map(|name| name.to_string());

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (name, trans) = (name, trans).merge()?;

        Ok(Self { name, trans })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) -> Entity {
        args.cmd.spawn((LevelButton, self.trans)).id()
    }
}
