use avian2d::prelude::*;
use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::{
    assets::{SpriteHandle, SpriteState},
    config::{Config, OBSTACLE_LAYER},
    state::{GameState, Obstacle, ObstacleSystem, Speed},
    util::ColliderMap,
};

pub struct DactylPlugin;

pub const DACTYL_SPAWN: usize = 2;

impl Plugin for DactylPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DactylHandles>()
            .insert_resource(DactylAnimationTimer(Timer::from_seconds(
                0.5,
                TimerMode::Repeating,
            )))
            .add_systems(OnEnter(SpriteState::Loaded), update_colliders)
            .add_systems(Startup, register_dactyl_spawn)
            .add_systems(
                FixedUpdate,
                animate_dactyl.run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Resource)]
struct DactylHandles {
    layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for DactylHandles {
    fn from_world(world: &mut World) -> Self {
        let cfg = world.resource::<Config>();
        let layout: TextureAtlasLayout = cfg.dactyl.into();
        let mut atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let handle = atlas_layouts.add(layout);
        DactylHandles { layout: handle }
    }
}

#[derive(Component)]
struct Dactyl;

#[derive(Resource, Default, Deref, DerefMut)]
struct DactylAnimationTimer(Timer);

fn register_dactyl_spawn(mut commands: Commands) {
    let id = commands.register_system(spawn_dactyl);
    commands.insert_resource(ObstacleSystem::<DACTYL_SPAWN>(id));
}

fn spawn_dactyl(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    handles: Res<DactylHandles>,
    colliders: Res<ColliderMap>,
    mut commands: Commands,
) {
    const Y_OPTS: [f32; 3] = [0.5, 0.1, 0.3];

    commands.spawn((
        Sprite::from_atlas_image(
            image.clone(),
            TextureAtlas {
                layout: handles.layout.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(
            cfg.size.x + cfg.dactyl.size.x as f32,
            cfg.size.y * Y_OPTS.choose(&mut rand::rng()).unwrap(),
            OBSTACLE_LAYER,
        ),
        TransformInterpolation,
        RigidBody::Kinematic,
        colliders.get_collider(&cfg.dactyl.rect(0)),
        Obstacle,
        Dactyl,
    ));
}

fn animate_dactyl(
    t: Res<Time<Fixed>>,
    cfg: Res<Config>,
    speed: Res<Speed>,
    colliders: Res<ColliderMap>,
    mut timer: ResMut<DactylAnimationTimer>,
    query: Single<(Entity, &mut Sprite, &mut Transform), With<Dactyl>>,
    mut commands: Commands,
) {
    let (ent, mut sprite, mut tf) = query.into_inner();

    let d = t.delta_secs() * speed.0 * 0.75;
    tf.translation.x -= d;

    if tf.translation.x < -(cfg.dactyl.size.x as f32) {
        commands.entity(ent).despawn();
        return;
    }

    timer.tick(t.delta());
    if timer.just_finished() {
        let atlas = sprite.texture_atlas.as_mut().unwrap();
        atlas.index = (atlas.index + 1) % cfg.dactyl.cols as usize;
        commands
            .entity(ent)
            .insert(colliders.get_collider(&cfg.dactyl.rect(atlas.index as u32)));
    }
}

fn update_colliders(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    mut colliders: ResMut<ColliderMap>,
    images: Res<Assets<Image>>,
    dactyls: Query<Entity, With<Dactyl>>,
    mut commands: Commands,
) {
    match images.get(image.id()) {
        Some(img) => {
            cfg.dactyl.add_colliders(img, &mut colliders);
            for ent in dactyls {
                let mut entity = commands.entity(ent);
                entity.insert(colliders.get(&cfg.dactyl.rect(0)).unwrap().clone());
            }
        }
        None => panic!(),
    }
}
