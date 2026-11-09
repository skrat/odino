use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{assets::SpriteHandle, config::*, state::*};

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundAtlas>();
        app.add_systems(Startup, spawn_ground);
        app.add_systems(
            FixedUpdate,
            (animate_ground.run_if(in_state(GameState::Running)),),
        );
    }
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct GroundTile;

#[derive(Resource, Deref)]
struct GroundAtlas(Handle<TextureAtlasLayout>);

impl FromWorld for GroundAtlas {
    fn from_world(world: &mut World) -> Self {
        let layout: TextureAtlasLayout = world.resource::<Config>().ground_sprite.into();
        let mut atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        GroundAtlas(atlas_layouts.add(layout))
    }
}

fn spawn_ground(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout: Res<GroundAtlas>,
    mut commands: Commands,
) {
    let vw = cfg.size.x;
    let num_tiles = (vw / cfg.ground_sprite.size.x as f32).ceil() as usize + 1;

    commands
        .spawn((
            Name::new("Ground"),
            Transform::from_translation(Vec3::new(0., 0., 80.)),
            Visibility::default(),
            Ground,
        ))
        .with_children(move |parent| {
            let mut x = 0.5 * cfg.ground_sprite.size.x as f32;
            for _ in 0..num_tiles {
                parent.spawn(tile_bundle(&cfg, image.clone(), layout.clone(), x));
                x += cfg.ground_sprite.size.x as f32;
            }
        });
}

fn tile_bundle(
    cfg: &Config,
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    x: f32,
) -> impl Bundle {
    (
        Sprite::from_atlas_image(
            image,
            TextureAtlas {
                layout,
                index: rand::random_range(0..(cfg.ground_sprite.cols as usize)),
            },
        ),
        Transform::from_xyz(x, 0., 0.),
        TransformInterpolation,
        GroundTile,
    )
}

fn animate_ground(
    t: Res<Time<Fixed>>,
    cfg: Res<Config>,
    speed: Res<Speed>,
    image: Res<SpriteHandle>,
    layout: Res<GroundAtlas>,
    tiles: Query<(Entity, &mut Transform), With<GroundTile>>,
    root: Single<Entity, With<Ground>>,
    mut commands: Commands,
) {
    let d = t.delta_secs() * speed.0;
    let max = tiles
        .iter()
        .fold(0.0, |acc, (_, tf)| tf.translation.x.max(acc));
    let root = root.into_inner();
    // Move all ground tiles
    for (ent, mut tf) in tiles {
        tf.translation.x -= d;
        if tf.translation.x < -(0.5 * cfg.ground_sprite.size.x as f32) {
            // Move tile to back (right) and randomize atlas index
            let x = max + cfg.ground_sprite.size.x as f32 - d;
            commands.entity(ent).despawn();
            commands.entity(root).with_children(|parent| {
                parent.spawn(tile_bundle(&cfg, image.clone(), layout.clone(), x));
            });
        }
    }
}
