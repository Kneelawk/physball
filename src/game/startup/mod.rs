use crate::game::state::GameState;
use bevy::asset::embedded_asset;
use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BallphysStartup;

impl Plugin for BallphysStartup {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "bevy_logo_dark.svg");

        app.add_systems(OnEnter(GameState::Splash), splash_screen);
    }
}

fn splash_screen(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
}
