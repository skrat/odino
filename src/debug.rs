use avian2d::debug_render::PhysicsGizmos;
#[cfg(feature = "dev")]
use bevy::color::palettes::css::{ORANGE, RED};
use bevy::prelude::*;

use crate::{
    state::GameState,
    trex::{Trex, TrexState},
};

pub struct DebugPlugin;

#[derive(Component)]
struct GameStateLabel(GameState);

#[derive(Component)]
struct TrexStateLabel(TrexState);

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_bottom_panel /* spawn_ground_line */,))
            .add_systems(Update, (update_gamestate_label, update_trexstate_label))
            .insert_gizmo_config(
                PhysicsGizmos {
                    aabb_color: Some(RED.with_alpha(0.5).into()),
                    collider_color: Some(ORANGE.with_alpha(0.5).into()),
                    ..default()
                },
                GizmoConfig::default(),
            );
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
        .with_children(|parent| {
            spawn_gamestate_label(parent);
            spawn_trexstate_label(parent);
        });
}

fn gamestate_str(s: &GameState) -> String {
    format!("Game state: {:?}", s)
}

fn spawn_gamestate_label(builder: &mut ChildSpawnerCommands) {
    builder.spawn(text_label(
        gamestate_str(&GameState::default()),
        GameStateLabel(GameState::default()),
    ));
}

fn update_gamestate_label(
    game_state: Res<State<GameState>>,
    query: Single<(&mut GameStateLabel, &mut Text)>,
) {
    let (mut s, mut t) = query.into_inner();
    if game_state.get() != &s.0 {
        *t = Text::new(gamestate_str(game_state.get()));
        *s = GameStateLabel(game_state.get().clone());
    }
}

fn trexstate_str(s: &TrexState) -> String {
    format!("Trex state: {:?}", s)
}

fn spawn_trexstate_label(builder: &mut ChildSpawnerCommands) {
    builder.spawn(text_label(
        trexstate_str(&TrexState::default()),
        TrexStateLabel(TrexState::default()),
    ));
}

fn update_trexstate_label(trex: Single<&Trex>, query: Single<(&mut TrexStateLabel, &mut Text)>) {
    let (mut s, mut t) = query.into_inner();
    if trex.state != s.0 {
        *s = TrexStateLabel(trex.state.clone());
    }
    let txt = format!("{}, JP={}", trexstate_str(&trex.state), trex.jump_power);
    *t = Text::new(txt);
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
