pub mod death;
pub mod finish_point;
pub mod index;
pub mod serial;

use crate::game::assets::fonts::FontNames;
use crate::game::assets::preload::Preloads;
use crate::game::levels::index::{LevelIndex, LevelIndexLoader, on_level_index_loaded};
use crate::game::levels::serial::SerialLevelLoader;
use crate::game::levels::serial::level::LevelBuildArgs;
use crate::game::state::AppState;
use bevy::asset::AssetLoadFailedEvent;
use bevy::prelude::*;
use serial::level::SerialLevel;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LevelIndex>()
            .init_asset::<SerialLevel>()
            .init_asset_loader::<LevelIndexLoader>()
            .init_asset_loader::<SerialLevelLoader>()
            .init_resource::<LevelLoadingLock>()
            .add_systems(Update, on_level_index_loaded)
            .add_systems(OnEnter(AppState::LoadingLevel), start_loading_level)
            .add_systems(
                Update,
                spawn_level.run_if(in_state(AppState::LoadingLevel).or(in_state(AppState::Game))),
            )
            .add_systems(
                Update,
                level_loading_error.run_if(in_state(AppState::LoadingLevel)),
            )
            .add_observer(respawn_level)
            .add_systems(OnExit(AppState::Game), (unselect_level, despawn_level));
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Event, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub struct LevelReadyEvent;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Event, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub struct LevelRestartEvent;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Resource, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash, Resource)]
pub struct SelectedLevel(pub String);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Resource, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash, Resource)]
pub struct LevelHandle(pub Handle<SerialLevel>);

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct LevelObject;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct PlayerSpawnPoint;

/// Mutable Resource to prevent level cancellation race condition
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Resource, Reflect)]
#[reflect(Debug, Default, Clone, Hash, Resource)]
pub enum LevelLoadingLock {
    #[default]
    NotLoading,
    Loading,
}

fn start_loading_level(
    mut cmd: Commands,
    level: Res<SelectedLevel>,
    assets: Res<AssetServer>,
    index: Res<LevelIndex>,
    mut broken_state: ResMut<NextState<AppState>>,
    mut level_lock: ResMut<LevelLoadingLock>,
) {
    let level = if let Some(level) = index.levels.get(&level.0) {
        level
    } else {
        error!("Attempted to load invalid level '{}'", &level.0);
        broken_state.set(AppState::MainMenu);
        return;
    };
    info!("Loading level '{}'", &level.name);
    cmd.insert_resource(LevelHandle(assets.load(&level.path)));
    *level_lock = LevelLoadingLock::Loading;
}

fn unselect_level(mut cmd: Commands) {
    cmd.remove_resource::<SelectedLevel>();
    cmd.remove_resource::<LevelHandle>();
}

fn despawn_level(mut cmd: Commands, query: Query<Entity, With<LevelObject>>) {
    despawn_level_impl(&mut cmd, query);
}

fn despawn_level_impl(cmd: &mut Commands, query: Query<Entity, With<LevelObject>>) {
    for level in query {
        cmd.entity(level).despawn();
    }
}

fn respawn_level(
    _on: On<LevelRestartEvent>,
    mut cmd: Commands,
    query: Query<Entity, With<LevelObject>>,
    asset_server: Res<AssetServer>,
    preloads: Res<Preloads>,
    fonts: Res<FontNames>,
    level_handle: Res<LevelHandle>,
    level_assets: Res<Assets<SerialLevel>>,
) {
    // TODO: Implement checkpoint system
    despawn_level_impl(&mut cmd, query);
    spawn_level_impl(
        &mut cmd,
        &asset_server,
        &preloads,
        &fonts,
        &level_handle,
        &level_assets,
        true,
    );
}

fn spawn_level(
    mut msg: MessageReader<AssetEvent<SerialLevel>>,
    mut cmd: Commands,
    query: Query<Entity, With<LevelObject>>,
    asset_server: Res<AssetServer>,
    preloads: Res<Preloads>,
    fonts: Res<FontNames>,
    level_handle: Option<Res<LevelHandle>>,
    level_assets: Res<Assets<SerialLevel>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut level_lock: ResMut<LevelLoadingLock>,
) {
    if let Some(level_handle) = level_handle
        && *level_lock == LevelLoadingLock::Loading
    {
        for e in msg.read() {
            if e.is_loaded_with_dependencies(&level_handle.0) {
                despawn_level_impl(&mut cmd, query);
                spawn_level_impl(
                    &mut cmd,
                    &asset_server,
                    &preloads,
                    &fonts,
                    &level_handle,
                    &level_assets,
                    true,
                );

                if **app_state != AppState::Game {
                    next_state.set(AppState::Game);
                    cmd.trigger(LevelReadyEvent);
                }

                *level_lock = LevelLoadingLock::NotLoading;

                msg.clear();
                return;
            }
        }
    }
}

fn spawn_level_impl(
    cmd: &mut Commands,
    assets: &AssetServer,
    preloads: &Preloads,
    fonts: &FontNames,
    level_handle: &LevelHandle,
    level_assets: &Assets<SerialLevel>,
    dyn_assets: bool,
) {
    let level = level_assets
        .get(&level_handle.0)
        .expect("Level handle invalid");
    level.spawn(&mut LevelBuildArgs {
        dyn_assets,
        cmd,
        assets,
        preloads,
        fonts,
    });
}

fn level_loading_error(
    mut msg: MessageReader<AssetLoadFailedEvent<SerialLevel>>,
    handle: Option<Res<LevelHandle>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(handle) = handle {
        for error in msg.read() {
            if error.id == handle.0.id() {
                next_state.set(AppState::LevelLoadingError);
            }
        }
    }
}
