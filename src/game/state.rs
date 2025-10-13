use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States, Reflect)]
pub enum AppState {
    #[default]
    Splash,
    MainMenu,
    // Loading would go here
    Game,
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}
