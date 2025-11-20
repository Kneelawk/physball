use crate::game::assets::asset_ref;
use crate::game::assets::fonts::FontNames;
use crate::game::assets::preload::Preloads;
use crate::game::levels::death::DeathCollider;
use crate::game::levels::finish_point::FinishPoint;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::{LevelObject, PlayerSpawnPoint};
use crate::{capture_result, type_expr};
use avian3d::prelude::*;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAlign, TextAtlas};
use kdl::{KdlDocument, KdlNode};
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
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialPlane {
    pub width: f32,
    pub length: f32,
    pub trans: Transform,
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
    pub pt: f32,
    pub font: Handle<Font>,
    pub align: SerialAlign,
    pub trans: Transform,
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
            .map(|node| SerialText::bind(node.clone(), load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let (spawn, finish, planes, texts) = (spawn, finish, planes, texts).merge()?;

        Ok(Self {
            spawn,
            finish,
            planes,
            texts,
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
    }
}

impl SerialPlane {
    pub fn bind(
        node: &KdlNode,
        _load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let size = node.must_get_number(0, &source);
        let size2 = node.get_number(1, &source);

        let ty = node
            .get_variant("type", SerialPlaneType::VARIANTS, &source)
            .map(|ty| ty.copied().unwrap_or_default());

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (size, size2, ty, trans) = (size, size2, ty, trans).merge()?;

        Ok(SerialPlane {
            width: size as f32,
            length: size2.unwrap_or(size) as f32,
            trans,
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
                    MeshMaterial3d(
                        args.assets
                            .add(type_expr!(StandardMaterial, Color::WHITE.into())),
                    ),
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
        node: KdlNode,
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
            .map(|asset| asset.unwrap_or(asset_ref::default_font(load_context)));

        let align = node
            .get_variant("align", SerialAlign::VARIANTS, &source)
            .map(|align| align.copied().unwrap_or_default());

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (text, pt, font, align, trans) = (text, pt, font, align, trans).merge()?;

        Ok(Self {
            text,
            pt: pt as f32,
            font,
            align,
            trans,
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
            MeshMaterial3d(args.assets.add(StandardMaterial {
                base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                emissive: LinearRgba::new(0.0, 10.0, 12.0, 1.0),
                ..default()
            })),
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
