use bevy::prelude::*;

use crate::{
    state::GameState,
    trex::{Trex, TrexState},
};

pub struct DebugPlugin;

#[derive(Component)]
struct GameStateLabel;

#[derive(Component)]
struct TrexStateLabel;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_bottom_panel /* spawn_ground_line */,))
            .add_systems(Update, (update_gamestate_label, update_trexstate_label));
    }
}

fn spawn_bottom_panel(mut commands: Commands) {
    commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Baseline,
            justify_content: JustifyContent::FlexEnd,
            padding: px(10).all(),
            row_gap: px(2),
            ..Default::default()
        })
        .with_children(|builder| {
            spawn_gamestate_label(builder);
            spawn_trexstate_label(builder);
        });
}

fn gamestate_str(s: &GameState) -> String {
    format!("Game state: {:?}", s)
}

fn spawn_gamestate_label(builder: &mut ChildSpawnerCommands) {
    builder.spawn(text_label(
        gamestate_str(&GameState::default()),
        GameStateLabel,
    ));
}

fn update_gamestate_label(
    game_state: Res<State<GameState>>,
    text: Single<&mut Text, With<GameStateLabel>>,
) {
    if !game_state.is_changed() {
        return;
    }
    let mut t = text.into_inner();
    *t = Text::new(gamestate_str(game_state.get()));
}

fn trexstate_str(s: &TrexState) -> String {
    format!("Trex state: {:?}", s)
}

fn spawn_trexstate_label(builder: &mut ChildSpawnerCommands) {
    builder.spawn(text_label(
        trexstate_str(&TrexState::default()),
        TrexStateLabel,
    ));
}

fn update_trexstate_label(trex: Single<&Trex>, text: Single<&mut Text, With<TrexStateLabel>>) {
    let mut t = text.into_inner();
    let s = format!("{}, JP={}", trexstate_str(&trex.state), trex.jump_power);
    *t = Text::new(s);
}

fn text_label<Marker: Component>(text: impl Into<String>, m: Marker) -> impl Bundle {
    (
        Text::new(text),
        TextColor(Color::WHITE),
        TextShadow {
            color: Color::BLACK,
            offset: Vec2::new(2.0, 2.0),
        },
        m,
    )
}

// fn spawn_ground_line(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     commands.spawn((
//         Mesh2d(meshes.add(Rectangle::new(DEFAULT_DIMENSIONS.x, 1.0))),
//         MeshMaterial2d(materials.add(Color::BLACK)),
//         Transform::from_translation(Vec3::new(DEFAULT_DIMENSIONS.x * 0.5, 0., 10.)),
//     ));
// }
