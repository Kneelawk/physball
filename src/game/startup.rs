use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BallphysStartup;

impl Plugin for BallphysStartup {
    fn build(&self, app: &mut App) {}
}
