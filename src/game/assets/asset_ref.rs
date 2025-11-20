use crate::game::assets::AssetType;
use crate::game::assets::builtin::{BUILTIN_HANDLES, BUILTIN_PREFIX};
use crate::game::assets::preload::{PRELOAD_FONT_TEXT, PRELOAD_PARTIALS, PRELOAD_PREFIX, Preloads};
use bevy::asset::{Asset, Handle, LoadContext, ParseAssetPathError};
use bevy::prelude::{Font, Reflect};
use std::borrow::Cow;
use thiserror::Error;

pub const DEFAULT_FONT: AssetRef<Font> = AssetRef::Preload(Cow::Borrowed(PRELOAD_FONT_TEXT));

pub fn load_in_loader<A: Asset + AssetType>(
    path: &str,
    load_context: &mut LoadContext,
) -> Result<Option<Handle<A>>, AssetRefError> {
    Ok(AssetRef::parse(path, load_context)?.resolve_in_loader(load_context))
}

#[derive(Debug, Clone, Reflect)]
pub enum AssetRef<T: Asset> {
    Preload(Cow<'static, str>),
    Handle(Handle<T>),
}

impl<T: Asset + AssetType> AssetRef<T> {
    pub fn parse(path: &str, load_context: &mut LoadContext) -> Result<AssetRef<T>, AssetRefError> {
        if let Some(builtin) = path.strip_prefix(BUILTIN_PREFIX) {
            BUILTIN_HANDLES[T::TYPE_NAME]
                .get(builtin)
                .map(|handle| handle.clone().typed::<T>())
                .ok_or_else(|| AssetRefError::MissingBuiltin(builtin.to_string()))
                .map(Self::Handle)
        } else if let Some(preload) = path.strip_prefix(PRELOAD_PREFIX) {
            Ok(Self::Preload(Cow::Owned(preload.to_string())))
        } else {
            let current_path = load_context.asset_path();
            Ok(Self::Handle(
                load_context.load(current_path.resolve_embed(path)?),
            ))
        }
    }

    pub fn resolve(&self, preloads: &Preloads) -> Option<Handle<T>> {
        match self {
            AssetRef::Preload(preload) => preloads.try_handle(preload),
            AssetRef::Handle(handle) => Some(handle.clone()),
        }
    }

    pub fn resolve_or_else(
        &self,
        preloads: &Preloads,
        resolver: impl FnOnce(&str) -> Handle<T>,
    ) -> Handle<T> {
        match self {
            AssetRef::Preload(preload) => preloads
                .try_handle(preload)
                .unwrap_or_else(|| resolver(preload)),
            AssetRef::Handle(handle) => handle.clone(),
        }
    }

    pub fn resolve_in_loader(&self, load_context: &mut LoadContext) -> Option<Handle<T>> {
        match self {
            AssetRef::Preload(preload) => PRELOAD_PARTIALS
                .lock()
                .unwrap()
                .try_lookup::<T>(preload)
                .map(|path| load_context.load(path)),
            AssetRef::Handle(handle) => Some(handle.clone()),
        }
    }
}

#[derive(Debug, Error)]
pub enum AssetRefError {
    #[error("Error parsing asset path {0}")]
    ParseAssetPath(#[from] ParseAssetPathError),
    #[error("No builtin with name '{0}'")]
    MissingBuiltin(String),
}
