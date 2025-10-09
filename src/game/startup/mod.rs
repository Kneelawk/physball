use crate::game::state::GameState;
use bevy::animation::{animated_field, AnimationTargetId};
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

#[derive(Debug, Clone, Resource, Reflect)]
struct SplashScreenData {
    name: Name,
    target_id: AnimationTargetId,
    graph: Handle<AnimationGraph>,
    node_index: AnimationNodeIndex,
}

fn splash_screen_setup(
    mut cmd: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let name = Name::new("bevy_logo_splash");
    let target_id = AnimationTargetId::from_name(&name);

    let mut clip = AnimationClip::default();

    clip.add_curve_to_target(target_id, AnimatableCurve::new(animated_field!()))
}

fn splash_screen(mut cmd: Commands, asset_server: Res<AssetServer>) {}
