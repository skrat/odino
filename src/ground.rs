use std::time::Duration;

use bevy::prelude::*;
use rand::{
    Rng, RngExt,
    distr::{Distribution, StandardUniform},
};

use crate::{constants::*, state::*};

const CACTI_SPAWN_RAND: std::ops::Range<f32> = 2.0..3.0;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_ground, setup_cacti).chain())
            .add_systems(
                Update,
                (
                    animate_ground.run_if(in_state(GameState::Running)),
                    spawn_cacti.run_if(in_state(GameState::Running)),
                    animate_cacti.run_if(in_state(GameState::Running)),
                ),
            );
    }
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct GroundTile;

#[derive(Resource)]
struct CactiHandles {
    tex: Handle<Image>,
    layout_sm: Handle<TextureAtlasLayout>,
    layout_md: Handle<TextureAtlasLayout>,
    layout_lg: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
enum Cacti {
    Sm,
    Md,
    Lg,
}

#[derive(Component)]
struct CactiSize {
    w: f32,
}

#[derive(Component, Deref, DerefMut)]
struct CactiTimer(Timer);

fn spawn_ground(
    cfg: Res<Config>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let tex = asset_server.load(SPRITE_PATH);
    let layout = atlas_layouts.add(cfg.ground_sprite);

    let vw = cfg.size.x;
    let num_tiles = (vw / cfg.ground_sprite.size.x as f32).ceil() as usize + 1;

    commands
        .spawn((
            Transform::from_translation(Vec3::new(0., 0., 80.)),
            Visibility::default(),
            Ground,
        ))
        .with_children(move |parent| {
            let mut x = 0.5 * cfg.ground_sprite.size.x as f32;
            for _ in 0..num_tiles {
                parent.spawn((
                    Sprite::from_atlas_image(
                        tex.clone(),
                        TextureAtlas {
                            layout: layout.clone(),
                            index: rand::random_range(0..(cfg.ground_sprite.cols as usize)),
                        },
                    ),
                    Transform::from_xyz(x, 0., 0.),
                    GroundTile,
                ));
                x += cfg.ground_sprite.size.x as f32;
            }
        });
}

fn animate_ground(
    t: Res<Time<Virtual>>,
    cfg: Res<Config>,
    s: Res<Speed>,
    q: Query<(&mut Transform, &mut Sprite), With<GroundTile>>,
) {
    let d = t.delta_secs() * s.0;
    let max = q.iter().fold(0.0, |acc, (tf, _)| tf.translation.x.max(acc));
    for (mut tf, mut sprite) in q {
        tf.translation.x -= d;
        if tf.translation.x < -(0.5 * cfg.ground_sprite.size.x as f32) {
            tf.translation.x = max + cfg.ground_sprite.size.x as f32 - d;
            let atlas = sprite.texture_atlas.as_mut().unwrap();
            atlas.index = rand::random_range(0..(cfg.ground_sprite.cols as usize));
        }
    }
}

fn setup_cacti(
    cfg: Res<Config>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let tex = asset_server.load(SPRITE_PATH);
    let layout_sm = atlas_layouts.add(cfg.cacti_sm);
    let layout_md = atlas_layouts.add(cfg.cacti_md);
    let layout_lg = atlas_layouts.add(cfg.cacti_lg);
    commands.insert_resource(CactiHandles {
        tex,
        layout_sm,
        layout_md,
        layout_lg,
    });
    commands.spawn(CactiTimer(Timer::from_seconds(
        rand::random_range(CACTI_SPAWN_RAND),
        TimerMode::Repeating,
    )));
}

fn spawn_cacti(
    t: Res<Time<Virtual>>,
    cfg: Res<Config>,
    mut timer: Query<&mut CactiTimer>,
    handle: Res<CactiHandles>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let mut tmr = timer.single_mut().unwrap();
    tmr.tick(t.delta());
    if tmr.just_finished() {
        commands.spawn(random_cacti(&cfg, &handle, &atlas_layouts));
        tmr.set_duration(Duration::from_secs_f32(rand::random_range(
            CACTI_SPAWN_RAND,
        )));
        tmr.reset();
    }
}

impl Distribution<Cacti> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cacti {
        match rng.random_range(0..3) {
            0 => Cacti::Sm,
            1 => Cacti::Md,
            2 => Cacti::Lg,
            _ => unreachable!(),
        }
    }
}

impl Cacti {
    fn pick(handle: &CactiHandles) -> (Cacti, Handle<TextureAtlasLayout>) {
        let c: Cacti = rand::random();
        match c {
            Cacti::Sm => (Cacti::Sm, handle.layout_sm.clone()),
            Cacti::Md => (Cacti::Md, handle.layout_md.clone()),
            Cacti::Lg => (Cacti::Lg, handle.layout_lg.clone()),
        }
    }
}

fn random_cacti(
    cfg: &Config,
    handle: &CactiHandles,
    atlas_layouts: &Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let (cacti, layout_handle) = Cacti::pick(handle);
    let layout = atlas_layouts.get(layout_handle.id()).unwrap();
    let size = match cacti {
        Cacti::Sm => cfg.cacti_sm.size,
        Cacti::Md => cfg.cacti_md.size,
        Cacti::Lg => cfg.cacti_lg.size,
    };
    (
        Sprite::from_atlas_image(
            handle.tex.clone(),
            TextureAtlas {
                layout: layout_handle,
                index: rand::random_range(0..layout.textures.len()),
            },
        ),
        Transform::from_xyz(
            cfg.size.x + size.x as f32,
            size.y as f32 * 0.5 - cfg.size.y * 0.05,
            0.,
        ),
        CactiSize { w: size.x as f32 },
        cacti,
    )
}

fn animate_cacti(
    t: Res<Time<Virtual>>,
    s: Res<Speed>,
    q: Query<(&mut Transform, &CactiSize, Entity), With<Cacti>>,
    mut commands: Commands,
) {
    let d = t.delta_secs() * s.0;
    for (mut tf, size, ent) in q {
        tf.translation.x -= d;
        if tf.translation.x < -size.w {
            commands.entity(ent).despawn();
        }
    }
}
