use crate::game::state::AppState;
use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::prelude::*;
use bevy_svg::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BallphysStartup;

impl Plugin for BallphysStartup {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "bevy_logo_dark.svg");

        app.add_systems(OnEnter(AppState::Splash), splash_screen)
            .add_systems(Update, splash_countdown.run_if(in_state(AppState::Splash)));
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct SplashCamera;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct SplashScreen;

#[derive(Debug, Clone, PartialEq, Eq, Resource, Deref, DerefMut, Reflect)]
#[reflect(Debug, Clone, PartialEq, Resource)]
struct SplashScreenTimer(Timer);

fn splash_screen(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((SplashCamera, Camera2d, DespawnOnExit(AppState::Splash)));

    cmd.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        DespawnOnExit(AppState::Splash),
        children![(
            Node {
                bottom: px(100),
                ..default()
            },
            Text::new("Made with")
        )],
    ));

    let svg = load_embedded_asset!(&*asset_server, "bevy_logo_dark.svg");
    cmd.spawn((
        SplashScreen,
        Svg2d(svg),
        Origin::Center,
        DespawnOnExit(AppState::Splash),
    ));

    cmd.insert_resource(SplashScreenTimer(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn splash_countdown(
    mut game_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashScreenTimer>,
) {
    if timer.tick(time.delta()).is_finished() {
        game_state.set(AppState::MainMenu);
    }
}
