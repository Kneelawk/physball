use crate::game::assets::asset_ref;
use crate::game::assets::asset_ref::AssetRefError;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Default)]
pub struct MaterialLoader;

impl AssetLoader for MaterialLoader {
    type Asset = StandardMaterial;
    type Settings = ();
    type Error = MaterialLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut str = String::new();
        reader.read_to_string(&mut str).await?;

        let json: StandardMaterialJson = serde_json::from_str(&str)?;

        json.bind(load_context)
    }

    fn extensions(&self) -> &[&str] {
        &["material.json"]
    }
}

#[derive(Debug, Error)]
pub enum MaterialLoadError {
    #[error("Error parsing asset ref {0}")]
    ParseAssetPath(#[from] AssetRefError),
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
    #[error("Json parse error {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct StandardMaterialJson {
    #[serde(default = "default_base_color")]
    pub base_color: Color,

    #[serde(default)]
    pub base_color_texture: Option<String>,

    #[serde(default = "default_emissive")]
    pub emissive: LinearRgba,

    #[serde(default)]
    pub emissive_texture: Option<String>,

    #[serde(default = "default_perceptual_roughness")]
    pub perceptual_roughness: f32,

    #[serde(default = "default_metallic")]
    pub metallic: f32,

    #[serde(default)]
    pub metallic_roughness_texture: Option<String>,

    #[serde(default)]
    pub cull_mode: CullModeJson,

    #[serde(default)]
    pub alpha_mode: AlphaModeJson,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CullModeJson {
    Front,
    #[default]
    Back,
    None,
}

impl From<CullModeJson> for Option<Face> {
    fn from(value: CullModeJson) -> Self {
        match value {
            CullModeJson::Front => Some(Face::Front),
            CullModeJson::Back => Some(Face::Back),
            CullModeJson::None => None,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AlphaModeJson {
    #[default]
    Opaque,
    Mask(f32),
    Blend,
    Premultiplied,
    AlphaToCoverage,
    Add,
    Multiply,
}

impl From<AlphaModeJson> for AlphaMode {
    fn from(value: AlphaModeJson) -> Self {
        match value {
            AlphaModeJson::Opaque => Self::Opaque,
            AlphaModeJson::Mask(cutoff) => Self::Mask(cutoff),
            AlphaModeJson::Blend => Self::Blend,
            AlphaModeJson::Premultiplied => Self::Premultiplied,
            AlphaModeJson::AlphaToCoverage => Self::AlphaToCoverage,
            AlphaModeJson::Add => Self::Add,
            AlphaModeJson::Multiply => Self::Multiply,
        }
    }
}

impl Default for StandardMaterialJson {
    fn default() -> Self {
        Self {
            base_color: default_base_color(),
            base_color_texture: None,
            emissive: default_emissive(),
            emissive_texture: None,
            perceptual_roughness: default_perceptual_roughness(),
            metallic: default_metallic(),
            metallic_roughness_texture: None,
            cull_mode: Default::default(),
            alpha_mode: Default::default(),
        }
    }
}

impl StandardMaterialJson {
    pub fn bind(
        self,
        load_context: &mut LoadContext,
    ) -> Result<StandardMaterial, MaterialLoadError> {
        Ok(StandardMaterial {
            base_color: self.base_color,
            base_color_texture: bind_handle(&self.base_color_texture, load_context)?,
            emissive: self.emissive,
            emissive_texture: bind_handle(&self.emissive_texture, load_context)?,
            perceptual_roughness: self.perceptual_roughness,
            metallic: self.metallic,
            metallic_roughness_texture: bind_handle(
                &self.metallic_roughness_texture,
                load_context,
            )?,
            cull_mode: self.cull_mode.into(),
            alpha_mode: self.alpha_mode.into(),
            ..default()
        })
    }
}

fn default_base_color() -> Color {
    Color::WHITE
}

fn default_emissive() -> LinearRgba {
    LinearRgba::BLACK
}

fn default_perceptual_roughness() -> f32 {
    0.5
}

fn default_metallic() -> f32 {
    0.0
}

fn bind_handle(
    to_bind: &Option<String>,
    load_context: &mut LoadContext,
) -> Result<Option<Handle<Image>>, MaterialLoadError> {
    Ok(to_bind
        .as_ref()
        .map(|tex| asset_ref::load_in_loader(tex, load_context))
        .map_or(Ok(None), |res| res.map(Some))?
        .flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_alpha_mode() {
        let json = json!({
            "alpha_mode": {
                "mask": 0.5
            }
        });

        let deserialized: StandardMaterialJson =
            serde_json::from_value(json).expect("error deserializing");

        assert_eq!(
            deserialized,
            StandardMaterialJson {
                alpha_mode: AlphaModeJson::Mask(0.5),
                ..default()
            }
        )
    }
}
