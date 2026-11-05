use avian2d::prelude::*;
use bevy::{camera::ScalingMode, prelude::*};

use crate::{
    assets::EmbeddedAssetsPlugin,
    constants::{Config, D100},
    debug::DebugPlugin,
    game::GameControlPlugin,
    ground::GroundPlugin,
    trex::TrexPlugin,
};

mod assets;
mod constants;
mod debug;
mod game;
mod ground;
mod state;
mod trex;
mod util;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PhysicsPlugins::default(),
        PhysicsDebugPlugin,
    ))
    .insert_resource(Gravity(Vec2::ZERO))
    .insert_resource(D100)
    .add_systems(Startup, (setup_camera, spawn_background))
    .add_plugins((
        EmbeddedAssetsPlugin,
        GameControlPlugin,
        TrexPlugin,
        GroundPlugin,
        DebugPlugin,
    ));

    #[cfg(feature = "dev")]
    app.insert_gizmo_config(
        PhysicsGizmos {
            aabb_color: Some(Color::srgb(1.0, 0., 0.)),
            ..default()
        },
        GizmoConfig::default(),
    );

    app.run();
}

fn setup_camera(cfg: Res<Config>, mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: cfg.size.x,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_translation(cfg.offset.extend(0.)),
    ));
}

fn spawn_background(
    cfg: Res<Config>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::from_size(cfg.size))),
        MeshMaterial2d(materials.add(Color::hsv(0.0, 0.0, 0.9))),
        Transform::from_translation(cfg.offset.extend(-100.)),
    ));
}
