use crate::game::assets::AssetType;
use crate::game::assets::builtin::{BUILTIN_HANDLES, BUILTIN_PREFIX};
use crate::game::assets::preload::{
    PRELOAD_FONT_TEXT, PRELOAD_MATERIAL_DEFAULT_PLANE, PRELOAD_MATERIAL_GLOW_TEXT,
    PRELOAD_PARTIALS, PRELOAD_PREFIX,
};
use bevy::asset::{Asset, Handle, LoadContext, ParseAssetPathError};
use bevy::prelude::*;
use thiserror::Error;

pub fn default_preload<A: Asset + AssetType>(
    key: &str,
    load_context: &mut LoadContext,
) -> Handle<A> {
    load(&format!("{PRELOAD_PREFIX}{key}"), load_context).unwrap_or_else(|err| {
        panic!(
            "error finding default preload '{}' (internal error): {}",
            key, err
        )
    })
}

pub fn default_font(load_context: &mut LoadContext) -> Handle<Font> {
    default_preload(PRELOAD_FONT_TEXT, load_context)
}

pub fn default_text_material(load_context: &mut LoadContext) -> Handle<StandardMaterial> {
    default_preload(PRELOAD_MATERIAL_GLOW_TEXT, load_context)
}

pub fn default_plane_material(load_context: &mut LoadContext) -> Handle<StandardMaterial> {
    default_preload(PRELOAD_MATERIAL_DEFAULT_PLANE, load_context)
}

pub fn load<A: Asset + AssetType>(
    path: &str,
    load_context: &mut LoadContext,
) -> Result<Handle<A>, AssetRefError> {
    if let Some(builtin) = path.strip_prefix(BUILTIN_PREFIX) {
        BUILTIN_HANDLES[A::TYPE_NAME]
            .get(builtin)
            .map(|handle| {
                handle
                    .clone()
                    .try_typed::<A>()
                    .expect("builtin registered with wrong type (internal error)")
            })
            .ok_or_else(|| {
                AssetRefError::MissingBuiltin(A::TYPE_NAME.to_string(), builtin.to_string())
            })
    } else if let Some(preload) = path.strip_prefix(PRELOAD_PREFIX) {
        PRELOAD_PARTIALS
            .lock()
            .unwrap()
            .try_lookup::<A>(preload)
            .map(|path| load_context.load::<A>(path))
            .ok_or_else(|| {
                AssetRefError::MissingPreload(A::TYPE_NAME.to_string(), preload.to_string())
            })
    } else {
        let current_path = load_context.asset_path();
        Ok(load_context.load(current_path.resolve_embed(path)?))
    }
}

#[derive(Debug, Error)]
pub enum AssetRefError {
    #[error("Error parsing asset path {0}")]
    ParseAssetPath(#[from] ParseAssetPathError),
    #[error("No builtin with type '{0}' and name '{1}'")]
    MissingBuiltin(String, String),
    #[error("No preload with type '{0}' and name '{1}'")]
    MissingPreload(String, String),
}
