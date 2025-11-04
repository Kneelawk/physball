use crate::game::assets::fonts::{BuiltinFonts, BuiltinFontsLoader, load_fonts, load_fonts_system};
use bevy::prelude::*;

#[derive(Default)]
pub struct BuiltinAssetsPlugin;

impl Plugin for BuiltinAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BuiltinFonts>()
            .init_asset_loader::<BuiltinFontsLoader>()
            .init_resource::<BuiltinAssetsState>()
            .add_systems(PreUpdate, load_fonts_system);
    }
}

pub fn load_all_builtins(cmd: &mut Commands, asset_server: &AssetServer) {
    load_fonts(cmd, asset_server);
}

#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Debug, Default, Clone, Resource)]
#[non_exhaustive]
pub struct BuiltinAssetsState {
    pub fonts: bool,
}

impl BuiltinAssetsState {
    pub fn is_done(&self) -> bool {
        self.fonts
    }
}
