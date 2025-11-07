use crate::game::assets::BuiltinAssetsState;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadDirectError};
use bevy::prelude::*;
use bevy_rich_text3d::TextRenderer;
use serde::Deserialize;
use thiserror::Error;

pub const FONTS_INDEX_PATH: &str = "fonts/index.json";

pub fn load_fonts(cmd: &mut Commands, asset_server: &AssetServer) {
    cmd.insert_resource(BuiltinFontsAsset(asset_server.load(FONTS_INDEX_PATH)));
}

pub fn load_fonts_system(
    mut msg: MessageReader<AssetEvent<BuiltinFonts>>,
    mut cmd: Commands,
    mut builtin_state: ResMut<BuiltinAssetsState>,
    handle: Option<Res<BuiltinFontsAsset>>,
    asset: Res<Assets<BuiltinFonts>>,
) {
    if let Some(handle) = handle {
        for e in msg.read() {
            if e.is_loaded_with_dependencies(&handle.0) {
                let fonts = asset.get(&handle.0).unwrap().clone();

                cmd.insert_resource(fonts.text_renderer.clone());
                cmd.insert_resource(fonts);
                *builtin_state = BuiltinAssetsState {
                    fonts: true,
                    ..*builtin_state
                };
                info!("Builtin fonts loaded.");
            }
        }
    }
}

#[derive(Default)]
pub struct BuiltinFontsLoader;

#[derive(Debug, Clone, Resource)]
pub struct BuiltinFontsAsset(Handle<BuiltinFonts>);

#[derive(Debug, Clone, Asset, Resource, TypePath)]
pub struct BuiltinFonts {
    pub title: Handle<Font>,
    pub text: Handle<Font>,
    pub title_name: String,
    pub text_name: String,
    text_renderer: TextRenderer,
}

#[derive(Debug, Clone, Deserialize)]
struct BuiltinFontsIndex {
    title: String,
    text: String,
}

impl AssetLoader for BuiltinFontsLoader {
    type Asset = BuiltinFonts;
    type Settings = ();
    type Error = BuiltinFontsLoadingError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut vec = vec![];
        reader.read_to_end(&mut vec).await?;
        let index: BuiltinFontsIndex = serde_json::from_slice(&vec)?;

        let title_font = load_context
            .loader()
            .immediate()
            .load::<Font>(index.title)
            .await?;
        let text_font = load_context
            .loader()
            .immediate()
            .load::<Font>(index.text)
            .await?;

        let locale = sys_locale::get_locale().unwrap_or("en_US".to_string());
        let db = cosmic_text::fontdb::Database::new();
        let mut system = cosmic_text::FontSystem::new_with_locale_and_db(locale, db);
        let title_id = system
            .db_mut()
            .load_font_source(cosmic_text::fontdb::Source::Binary(
                title_font.get().data.clone(),
            ))[0];
        let text_id = system
            .db_mut()
            .load_font_source(cosmic_text::fontdb::Source::Binary(
                text_font.get().data.clone(),
            ))[0];
        let title_name = system.db().face(title_id).unwrap().families[0].0.clone();
        let text_name = system.db().face(text_id).unwrap().families[0].0.clone();

        info!("Title font name: '{}'", &title_name);
        info!("Text font name: '{}'", &text_name);

        let title = load_context.add_loaded_labeled_asset("title", title_font);
        let text = load_context.add_loaded_labeled_asset("text", text_font);

        Ok(BuiltinFonts {
            title,
            text,
            title_name,
            text_name,
            text_renderer: TextRenderer::new(system),
        })
    }
}

#[derive(Debug, Error)]
pub enum BuiltinFontsLoadingError {
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error {0}")]
    Json(#[from] serde_json::Error),
    #[error("Error loading font {0}")]
    Dependency(#[from] LoadDirectError),
}
