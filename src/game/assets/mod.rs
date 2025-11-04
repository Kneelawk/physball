pub mod fonts;
pub mod preload;

use crate::game::assets::preload::{Preloads, PreloadsLoader, load_preloads, load_preloads_system};
use bevy::prelude::*;
use fonts::{BuiltinFonts, BuiltinFontsLoader, load_fonts, load_fonts_system};

#[derive(Default)]
pub struct BuiltinAssetsPlugin;

impl Plugin for BuiltinAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BuiltinFonts>()
            .init_asset::<Preloads>()
            .init_asset_loader::<BuiltinFontsLoader>()
            .init_asset_loader::<PreloadsLoader>()
            .init_resource::<BuiltinAssetsState>()
            .add_systems(PreUpdate, (load_fonts_system, load_preloads_system));
    }
}

pub fn load_all_builtins(cmd: &mut Commands, asset_server: &AssetServer) {
    load_fonts(cmd, asset_server);
    load_preloads(cmd, asset_server);
}

#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Debug, Default, Clone, Resource)]
#[non_exhaustive]
pub struct BuiltinAssetsState {
    pub fonts: bool,
    pub preloads: bool,
}

impl BuiltinAssetsState {
    pub fn is_done(&self) -> bool {
        self.fonts && self.preloads
    }
}
