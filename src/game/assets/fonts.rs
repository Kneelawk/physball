use bevy::prelude::*;
use bevy_rich_text3d::TextRenderer;
use cosmic_text::fontdb::{ID, Source};
use sha2::digest::array::Array;
use sha2::digest::consts::U32;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Deref, DerefMut, Resource, Reflect)]
#[reflect(Debug, Default, Clone, Resource)]
pub struct FontNames(HashMap<AssetId<Font>, String>);

#[derive(Debug, Default, Clone, Deref, DerefMut, Resource, Reflect)]
#[reflect(Debug, Default, Clone)]
pub struct LoadedFonts(#[reflect(ignore)] HashMap<Array<u8, U32>, ID>);

pub fn insert_fonts(
    mut msg: MessageReader<AssetEvent<Font>>,
    fonts: Res<Assets<Font>>,
    mut text_renderer: ResMut<TextRenderer>,
    mut names: ResMut<FontNames>,
    mut loaded: ResMut<LoadedFonts>,
) {
    for e in msg.read() {
        if let AssetEvent::LoadedWithDependencies { id: asset_id } = e {
            let font = fonts.get(*asset_id).expect("loaded font is missing");

            let digest = Sha256::digest(&font.data[..]);
            if loaded.contains_key(&digest.0) {
                let font_id = loaded[&digest.0];
                if let Some(face) = text_renderer.lock().db().face(font_id) {
                    let name = face.families[0].0.clone();
                    info!("Font already loaded {}", &name);
                    names.insert(*asset_id, name);
                } else {
                    warn!(
                        "Loaded font marked as already loaded, but its associated id was not found in the database"
                    );

                    let new_id = text_renderer
                        .lock()
                        .db_mut()
                        .load_font_source(Source::Binary(font.data.clone()))[0];
                    loaded.insert(digest, new_id);
                    let name = text_renderer.lock().db().face(new_id).unwrap().families[0]
                        .0
                        .clone();
                    info!("New font name: {}", &name);
                    names.insert(*asset_id, name);
                }
            } else {
                let font_id = text_renderer
                    .lock()
                    .db_mut()
                    .load_font_source(Source::Binary(font.data.clone()))[0];
                loaded.insert(digest, font_id);
                let name = text_renderer.lock().db().face(font_id).unwrap().families[0]
                    .0
                    .clone();
                info!("Inserted new font with name {}", &name);
                names.insert(*asset_id, name);
            }
        } else if let AssetEvent::Removed { id } = e {
            names.remove(id);
        }
    }
}
