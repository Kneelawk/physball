use crate::game::assets::preload::{PRELOAD_FONT_TEXT, PreloadType, Preloads};
use bevy::asset::{Asset, Handle, LoadContext, ParseAssetPathError};
use bevy::prelude;
use bevy::prelude::{Font, Reflect};
use std::borrow::Cow;

pub const DEFAULT_FONT: AssetRef<Font> = AssetRef::Preload(Cow::Borrowed(PRELOAD_FONT_TEXT));

#[derive(Debug, Clone, Reflect)]
pub enum AssetRef<T: Asset> {
    Preload(Cow<'static, str>),
    Handle(Handle<T>),
}

impl<T: Asset> AssetRef<T> {
    pub fn parse(
        path: &str,
        load_context: &mut LoadContext,
    ) -> prelude::Result<AssetRef<T>, ParseAssetPathError> {
        if let Some(preload) = path.strip_prefix("preload:") {
            Ok(Self::Preload(Cow::Owned(preload.to_string())))
        } else {
            let current_path = load_context.asset_path();
            Ok(Self::Handle(
                load_context.load(current_path.resolve_embed(path)?),
            ))
        }
    }
}

impl<T: Asset + PreloadType> AssetRef<T> {
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
}
