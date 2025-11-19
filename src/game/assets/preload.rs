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

pub const PRELOAD_TYPE_SCENE: &str = "scene";
pub const PRELOAD_TYPE_FONT: &str = "font";

pub const PRELOAD_SCENE_PHYSBALL: &str = "physball";
pub const PRELOAD_SCENE_LEVEL_END: &str = "level-end";
pub const PRELOAD_FONT_TITLE: &str = "title";
pub const PRELOAD_FONT_TEXT: &str = "text";

lazy_static! {
    pub static ref ASSET_TYPES: HashMap<String, TypeId> = {
        let mut tys = HashMap::new();
        tys.insert(PRELOAD_TYPE_SCENE.to_string(), TypeId::of::<Scene>());
        tys.insert(PRELOAD_TYPE_FONT.to_string(), TypeId::of::<Font>());
        tys
    };
    pub static ref REQURED_PRELOADS: HashMap<String, String> = {
        let mut reqs = HashMap::new();
        reqs.insert(
            PRELOAD_SCENE_PHYSBALL.to_string(),
            PRELOAD_TYPE_SCENE.to_string(),
        );
        reqs.insert(
            PRELOAD_SCENE_LEVEL_END.to_string(),
            PRELOAD_TYPE_SCENE.to_string(),
        );
        reqs.insert(
            PRELOAD_FONT_TITLE.to_string(),
            PRELOAD_TYPE_FONT.to_string(),
        );
        reqs.insert(PRELOAD_FONT_TEXT.to_string(), PRELOAD_TYPE_FONT.to_string());
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

pub trait PreloadType {
    const PRELOAD_TYPE_NAME: &'static str;
}

impl PreloadType for Scene {
    const PRELOAD_TYPE_NAME: &'static str = PRELOAD_TYPE_SCENE;
}

impl PreloadType for Font {
    const PRELOAD_TYPE_NAME: &'static str = PRELOAD_TYPE_FONT;
}

#[derive(Default)]
pub struct PreloadsLoader;

#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Debug, Clone, Resource)]
pub struct PreloadsAsset(Handle<Preloads>);

#[derive(Debug, Clone, Asset, Resource, Deref, Reflect)]
#[reflect(Debug, Clone, Resource)]
pub struct Preloads(HashMap<String, HashMap<String, Preload>>);

impl Preloads {
    pub fn handle<A: Asset + PreloadType>(&self, asset_name: &str) -> Handle<A> {
        self[A::PRELOAD_TYPE_NAME][asset_name]
            .handle
            .clone()
            .typed()
    }

    pub fn try_handle<A: Asset + PreloadType>(&self, asset_name: &str) -> Option<Handle<A>> {
        self[A::PRELOAD_TYPE_NAME]
            .get(asset_name)
            .map(|preload| preload.handle.clone().typed())
    }

    pub fn physball(&self) -> Handle<Scene> {
        self.handle(PRELOAD_SCENE_PHYSBALL)
    }

    pub fn level_end(&self) -> Handle<Scene> {
        self.handle(PRELOAD_SCENE_LEVEL_END)
    }

    pub fn title_font(&self) -> Handle<Font> {
        self.handle(PRELOAD_FONT_TITLE)
    }

    pub fn text_font(&self) -> Handle<Font> {
        self.handle(PRELOAD_FONT_TEXT)
    }
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct Preload {
    pub ty: String,
    pub handle: UntypedHandle,
}

#[derive(Debug, Clone, Deserialize)]
struct PreloadsJson {
    preloads: Vec<PreloadJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct PreloadJson {
    name: String,
    #[serde(rename = "type")]
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
        let index: PreloadsJson = serde_json::from_slice(&vec)?;

        let mut reqs = REQURED_PRELOADS.clone();
        let mut preloads = HashMap::new();
        for asset_type in ASSET_TYPES.keys() {
            preloads.insert(asset_type.clone(), HashMap::new());
        }

        for preload_json in index.preloads {
            if let Some(req_ty) = reqs.remove(&preload_json.name)
                && preload_json.ty != req_ty
            {
                return Err(PreloadsLoadingError::WrongPreloadType(preload_json.ty, req_ty));
            }

            let ty =
                *ASSET_TYPES
                    .get(&preload_json.ty)
                    .ok_or(PreloadsLoadingError::UnknownAssetType {
                        ty: preload_json.ty.clone(),
                    })?;
            let preload = Preload {
                ty: preload_json.ty.clone(),
                handle: load_context
                    .loader()
                    .with_dynamic_type(ty)
                    .load(preload_json.path),
            };
            preloads.get_mut(&preload.ty).unwrap().insert(preload_json.name, preload);
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
