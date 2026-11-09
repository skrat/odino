use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::{
    Rng, RngExt,
    distr::{Distribution, StandardUniform, weighted::WeightedIndex},
};

use crate::{
    assets::{SpriteHandle, SpriteState},
    config::Config,
    state::{GameState, Obstacle, Speed},
    util::ColliderMap,
};

pub struct CactiPlugin;

impl Plugin for CactiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CactiHandles>()
            .add_systems(Startup, spawn_cacti_timer)
            .add_systems(OnEnter(SpriteState::Loaded), update_colliders)
            .add_systems(
                FixedUpdate,
                (
                    spawn_cacti.run_if(in_state(GameState::Running)),
                    animate_cacti.run_if(in_state(GameState::Running)),
                ),
            );
    }
}

#[derive(Resource)]
struct CactiHandles {
    layout_sm: Handle<TextureAtlasLayout>,
    layout_md: Handle<TextureAtlasLayout>,
    layout_lg: Handle<TextureAtlasLayout>,
}

impl FromWorld for CactiHandles {
    fn from_world(world: &mut World) -> Self {
        let cfg = world.resource::<Config>();
        let lsm: TextureAtlasLayout = cfg.cacti_sm.into();
        let lmd: TextureAtlasLayout = cfg.cacti_md.into();
        let llg: TextureAtlasLayout = cfg.cacti_lg.into();
        let mut atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout_sm = atlas_layouts.add(lsm);
        let layout_md = atlas_layouts.add(lmd);
        let layout_lg = atlas_layouts.add(llg);
        CactiHandles {
            layout_sm,
            layout_md,
            layout_lg,
        }
    }
}

#[derive(Component, Clone)]
enum Cacti {
    Sm,
    Md,
    Lg,
}

#[derive(Component)]
struct CactiTotalSize {
    w: f32,
}

#[derive(Component, Deref)]
struct CactiRect(URect);

#[derive(Component, Deref, DerefMut)]
struct CactiTimer(Timer);

const CACTI_SPAWN_RAND: std::ops::Range<f32> = 1.0..3.0;

fn spawn_cacti_timer(mut commands: Commands) {
    commands.spawn(CactiTimer(Timer::from_seconds(
        rand::random_range(CACTI_SPAWN_RAND),
        TimerMode::Repeating,
    )));
}

fn spawn_cacti(
    t: Res<Time<Fixed>>,
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    colliders: Res<ColliderMap>,
    mut timer: Query<&mut CactiTimer>,
    handle: Res<CactiHandles>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    commands: Commands,
) {
    let mut tmr = timer.single_mut().unwrap();
    tmr.tick(t.delta());
    if tmr.just_finished() {
        random_cacti(
            &cfg,
            image.clone(),
            &handle,
            &colliders,
            &atlas_layouts,
            commands,
        );
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
        const CHOICES: [Cacti; 3] = [Cacti::Sm, Cacti::Md, Cacti::Lg];
        const WEIGHTS: [usize; 3] = [3, 3, 1];

        let dist = WeightedIndex::new(&WEIGHTS).unwrap();
        let mut rng = rand::rng();
        let c = &CHOICES[dist.sample(&mut rng)];

        match c {
            Cacti::Sm => (Cacti::Sm, handle.layout_sm.clone()),
            Cacti::Md => (Cacti::Md, handle.layout_md.clone()),
            Cacti::Lg => (Cacti::Lg, handle.layout_lg.clone()),
        }
    }
}

fn random_cacti(
    cfg: &Config,
    image: Handle<Image>,
    handle: &CactiHandles,
    colliders: &ColliderMap,
    atlas_layouts: &Assets<TextureAtlasLayout>,
    mut commands: Commands,
) {
    let (cacti, layout_handle) = Cacti::pick(handle);
    let layout = atlas_layouts.get(layout_handle.id()).unwrap();
    let single_size = match cacti {
        Cacti::Sm => cfg.cacti_sm.size,
        Cacti::Md => cfg.cacti_md.size,
        Cacti::Lg => cfg.cacti_lg.size,
    };
    let atlas = TextureAtlas {
        layout: layout_handle,
        index: rand::random_range(0..layout.textures.len()),
    };
    let sprite = Sprite::from_atlas_image(image, atlas.clone());
    let cnt = match cacti {
        Cacti::Sm => rand::random_range(1..=3),
        Cacti::Md => rand::random_range(1..=2),
        Cacti::Lg => 1,
    };
    let total_width = (single_size.x * cnt) as f32;
    commands
        .spawn((
            Name::new("Cacti"),
            Visibility::default(),
            Transform::from_xyz(
                cfg.size.x + total_width,
                single_size.y as f32 * 0.5 - cfg.size.y * 0.05,
                0.,
            ),
            TransformInterpolation,
            RigidBody::Kinematic,
            CactiTotalSize { w: total_width },
            Obstacle,
        ))
        .with_children(|parent| {
            let rect = atlas.texture_rect(atlas_layouts).unwrap();
            let collider = colliders
                .get(&rect)
                .and_then(|x| Some(x.clone()))
                .unwrap_or_else(|| {
                    println!("MISSED CMAP {:?}", rect);
                    Collider::rectangle(single_size.x as f32, single_size.y as f32)
                });
            let mut x = -0.5 * total_width + 0.5 * single_size.x as f32;
            for _ in 0..cnt {
                parent.spawn((
                    sprite.clone(),
                    Transform::from_xyz(x, 0., 0.),
                    collider.clone(),
                    CactiRect(rect),
                ));
                x += single_size.x as f32;
            }
        });
}

fn animate_cacti(
    t: Res<Time<Fixed>>,
    s: Res<Speed>,
    q: Query<(&mut Transform, &CactiTotalSize, Entity)>,
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

fn update_colliders(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    mut colliders: ResMut<ColliderMap>,
    images: Res<Assets<Image>>,
    cacti: Query<(Entity, &CactiRect)>,
    mut commands: Commands,
) {
    match images.get(image.id()) {
        Some(img) => {
            cfg.cacti_sm.add_colliders(img, &mut colliders);
            cfg.cacti_md.add_colliders(img, &mut colliders);
            cfg.cacti_lg.add_colliders(img, &mut colliders);

            for (e, rect) in cacti {
                let mut entity = commands.entity(e);
                entity.insert(colliders.get(&rect).unwrap().clone());
            }
        }
        None => panic!(),
    }
}
