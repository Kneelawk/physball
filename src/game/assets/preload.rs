use crate::game::assets::BuiltinAssetsState;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::any::TypeId;
use std::collections::HashMap;
use thiserror::Error;

pub const PRELOAD_INDEX_PATH: &str = "preload/index.json";

lazy_static! {
    pub static ref ASSET_TYPES: HashMap<String, TypeId> = {
        let mut tys = HashMap::new();
        tys.insert("scene".to_string(), TypeId::of::<Scene>());
        tys
    };
    pub static ref REQURED_PRELOADS: HashMap<String, String> = {
        let mut reqs = HashMap::new();
        reqs.insert("level-end".to_string(), "scene".to_string());
        reqs
    };
}

pub fn load_preloads(cmd: &mut Commands, asset_server: &AssetServer) {
    cmd.insert_resource(PreloadsAsset(asset_server.load(PRELOAD_INDEX_PATH)));
}

pub fn load_preloads_system(
    mut msg: MessageReader<AssetEvent<Preloads>>,
    mut cmd: Commands,
    mut builtin_state: ResMut<BuiltinAssetsState>,
    handle: Option<Res<PreloadsAsset>>,
    asset: Res<Assets<Preloads>>,
) {
    if let Some(handle) = handle {
        for e in msg.read() {
            if e.is_loaded_with_dependencies(&handle.0) {
                cmd.insert_resource(asset.get(&handle.0).unwrap().clone());
                *builtin_state = BuiltinAssetsState {
                    preloads: true,
                    ..*builtin_state
                };
                info!("Preloads loaded.");
            }
        }
    }
}

#[derive(Default)]
pub struct PreloadsLoader;

#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Debug, Clone, Resource)]
pub struct PreloadsAsset(Handle<Preloads>);

#[derive(Debug, Clone, Asset, Resource, Deref, Reflect)]
#[reflect(Debug, Clone, Resource)]
pub struct Preloads(HashMap<String, Preload>);

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct Preload {
    pub ty: String,
    pub handle: UntypedHandle,
}

#[derive(Debug, Clone, Deserialize)]
struct PreloadJson {
    ty: String,
    path: String,
}

impl AssetLoader for PreloadsLoader {
    type Asset = Preloads;
    type Settings = ();
    type Error = PreloadsLoadingError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut vec = vec![];
        reader.read_to_end(&mut vec).await?;
        let index: HashMap<String, PreloadJson> = serde_json::from_slice(&vec)?;

        let mut reqs = REQURED_PRELOADS.clone();
        let mut preloads = HashMap::new();
        for (key, preload) in index {
            if let Some(req_ty) = reqs.remove(&key)
                && preload.ty != req_ty
            {
                return Err(PreloadsLoadingError::WrongPreloadType(preload.ty, req_ty));
            }

            let ty =
                *ASSET_TYPES
                    .get(&preload.ty)
                    .ok_or(PreloadsLoadingError::UnknownAssetType {
                        ty: preload.ty.clone(),
                    })?;
            let preload = Preload {
                ty: preload.ty,
                handle: load_context
                    .loader()
                    .with_dynamic_type(ty)
                    .load(preload.path),
            };
            preloads.insert(key, preload);
        }

        if !reqs.is_empty() {
            return Err(PreloadsLoadingError::MissingPreloads(
                reqs.keys().cloned().collect(),
            ));
        }

        Ok(Preloads(preloads))
    }
}

#[derive(Debug, Error)]
pub enum PreloadsLoadingError {
    #[error("Unknown asset type '{}', known asset types are {:?}", .ty, ASSET_TYPES.keys().collect::<Vec<_>>())]
    UnknownAssetType { ty: String },
    #[error("Missing required preloads {0:?}")]
    MissingPreloads(Vec<String>),
    #[error("Preload has wrong type '{0}', expected '{1}'")]
    WrongPreloadType(String, String),
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error {0}")]
    Json(#[from] serde_json::Error),
}
