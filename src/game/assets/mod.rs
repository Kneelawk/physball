pub mod fonts;
pub mod preload;
mod materials;

use crate::game::assets::fonts::{FontNames, LoadedFonts, insert_fonts};
use crate::game::assets::preload::{Preloads, PreloadsLoader, load_preloads, load_preloads_system};
use crate::game::levels::index::load_level_index;
use bevy::app::MainScheduleOrder;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, ScheduleLabel)]
pub struct AssetProcess;

#[derive(Default)]
pub struct BuiltinAssetsPlugin;

impl Plugin for BuiltinAssetsPlugin {
    fn build(&self, app: &mut App) {
        let asset_process = Schedule::new(AssetProcess);

        app.add_schedule(asset_process)
            .world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, AssetProcess);

        app.init_asset::<Preloads>()
            .init_asset_loader::<PreloadsLoader>()
            .init_resource::<BuiltinAssetsState>()
            .init_resource::<FontNames>()
            .init_resource::<LoadedFonts>()
            .add_systems(AssetProcess, (insert_fonts, load_preloads_system));
    }
}

pub fn load_all_builtins(cmd: &mut Commands, asset_server: &AssetServer) {
    load_preloads(cmd, asset_server);
    load_level_index(cmd, asset_server);
}

#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Debug, Default, Clone, Resource)]
#[non_exhaustive]
pub struct BuiltinAssetsState {
    pub preloads: bool,
    pub level_index: bool,
}

impl BuiltinAssetsState {
    pub fn is_done(&self) -> bool {
        self.preloads && self.level_index
    }
}
