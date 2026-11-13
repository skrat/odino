use bevy::{ecs::system::SystemId, prelude::*};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Init,
    Running,
    Paused,
    GameOver,
}

#[derive(Resource, Deref)]
pub struct Speed(pub f32);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Score(pub f32);

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct ObstacleSystem {
    pub weight: usize,
    pub system_id: SystemId,
    pub min_score: Option<f32>,
    pub delay_min: u64,
    pub delay_max: u64,
}

pub fn not_in_state<S: States>(state: S) -> impl FnMut(Option<Res<State<S>>>) -> bool + Clone {
    move |current_state: Option<Res<State<S>>>| match current_state {
        Some(current_state) => *current_state != state,
        None => true,
    }
}
