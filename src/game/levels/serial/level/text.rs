use crate::game::assets::asset_ref;
use crate::game::levels::LevelObject;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::serial::level::{DEFAULT_TEXT_PT, LevelBuildArgs};
use bevy::asset::{Handle, LoadContext};
use bevy::math::Vec2;
use bevy::mesh::Mesh3d;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude;
use bevy::prelude::{Font, Reflect, Transform};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAlign};
use kdl::KdlNode;
use std::sync::Arc;
use strum::VariantArray;
use tracing::warn;

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

impl SerialText {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> prelude::Result<Self, KdlBindError> {
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
