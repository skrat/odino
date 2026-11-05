use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{assets::*, constants::*, state::*, util::*};

pub struct TrexPlugin;

impl Plugin for TrexPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_trex)
            .add_systems(
                Update,
                (
                    animate_trex.run_if(in_state(GameState::Running)),
                    jump_on_key.run_if(in_state(GameState::Running)),
                    jumping_trex.run_if(
                        in_state(GameState::Running).and_then(trex_in_state(TrexState::Jumping)),
                    ),
                    detect_collisions.run_if(in_state(GameState::Running)),
                    check_image_loaded.run_if(in_state(ImageState::Init)),
                ),
            )
            .insert_resource(JumpKeyDown(false))
            .init_resource::<ImageHandle>()
            .init_state::<ImageState>()
            .add_systems(OnEnter(ImageState::Loaded), update_collider)
            .init_resource::<KeySoundEffect>();

        #[cfg(feature = "dev")]
        app.add_systems(
            Update,
            bevy::dev_tools::states::log_transitions::<ImageState>,
        );
    }
}

pub fn trex_in_state(state: TrexState) -> impl FnMut(Option<Single<&Trex>>) -> bool {
    move |trex: Option<Single<&Trex>>| match trex {
        Some(trex) => trex.state == state,
        None => false,
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum TrexState {
    #[default]
    Running,
    Jumping,
}

#[derive(Component, Default)]
pub struct Trex {
    pub state: TrexState,
    pub jumped_at: f64,
    pub jump_power: f32,
}

#[derive(Resource, Default)]
struct JumpKeyDown(bool);

#[derive(Component)]
struct AnimationIndices {
    start: usize,
    end: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Resource, Deref)]
struct KeySoundEffect {
    handle: Handle<AudioSource>,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum ImageState {
    #[default]
    Init,
    Loaded,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct ImageHandle(Option<Handle<Image>>);

impl FromWorld for KeySoundEffect {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        KeySoundEffect {
            handle: asset_server.load(BUTTON_PRESS_PATH),
        }
    }
}

fn spawn_trex(
    cfg: Res<Config>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut image: ResMut<ImageHandle>,
    mut commands: Commands,
) {
    let tex = asset_server.load::<Image>(SPRITE_PATH);
    *image = ImageHandle(Some(tex.clone()));

    let layout = atlas_layouts.add(cfg.trex_sprite);
    let tile_start = 2;
    let tile_end = 3;

    let pos = Vec3::new(50., cfg.trex_sprite.size.y as f32 * 0.35, 100.);
    // let collider_points = points_for_convex_hull(*tex, rect, None);

    commands.spawn((
        Transform::from_translation(pos),
        Visibility::default(),
        children![(
            Name::new("T-Rex"),
            Sprite::from_atlas_image(
                tex,
                TextureAtlas {
                    layout,
                    index: tile_start,
                },
            ),
            AnimationIndices {
                start: tile_start,
                end: tile_end,
            },
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Transform::from_translation(Vec3::ZERO),
            RigidBody::Kinematic,
            Collider::rectangle(cfg.trex_sprite.size.x as f32, cfg.trex_sprite.size.y as f32),
            CollidingEntities::default(),
            Trex::default(),
        )],
    ));
}

fn animate_trex(
    t: Res<Time<Virtual>>,
    q: Single<(&Trex, &AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    let (trex, indices, mut timer, mut sprite) = q.into_inner();
    timer.tick(t.delta());

    if trex.state == TrexState::Running
        && timer.just_finished()
        && let Some(atlas) = &mut sprite.texture_atlas
    {
        atlas.index = if atlas.index == indices.end {
            indices.start
        } else {
            atlas.index + 1
        };
    }
}

fn jumping_trex(
    t: Res<Time<Virtual>>,
    cfg: Res<Config>,
    down: Res<JumpKeyDown>,
    q: Single<(&mut Trex, &mut Transform, &AnimationIndices, &mut Sprite)>,
) {
    let (mut trex, mut tf, indices, mut sprite) = q.into_inner();
    let d = trex.jump_power * 0.5;
    let p = 1.0 - (0.5 - (t.elapsed_secs_f64() - trex.jumped_at) as f32 / d).abs() * 2.0;
    let e = EaseFunction::QuadraticOut.sample(p).unwrap_or(0.);
    tf.translation.y = cfg.size.y * 0.75 * trex.jump_power * e;
    if t.elapsed_secs_f64() - trex.jumped_at >= d as f64 {
        if down.0 {
            trex.jumped_at = t.elapsed_secs_f64();
            trex.jump_power = cfg.init_jump_power;
        } else {
            trex.state = TrexState::Running;
            let atlas = sprite.texture_atlas.as_mut().unwrap();
            atlas.index = indices.start;
        }
    }
}

fn jump_on_key(
    t: Res<Time<Virtual>>,
    cfg: Res<Config>,
    input: Res<ButtonInput<KeyCode>>,
    mut down: ResMut<JumpKeyDown>,
    query: Single<(&mut Trex, &mut Sprite)>,
    sound_effect: Res<KeySoundEffect>,
    mut commands: Commands,
) {
    let (mut trex, mut sprite) = query.into_inner();
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::ArrowUp) {
        match trex.state {
            TrexState::Running => {
                let atlas = sprite.texture_atlas.as_mut().unwrap();
                trex.state = TrexState::Jumping;
                trex.jumped_at = t.elapsed_secs_wrapped_f64();
                trex.jump_power = cfg.init_jump_power;
                down.0 = true;
                atlas.index = 0;
                commands.spawn((
                    AudioPlayer::new(sound_effect.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
            _ => (),
        }
    }
    if down.0 {
        trex.jump_power += t.delta().as_secs_f32() * 1.2;
    }
    if input.just_released(KeyCode::Space) || input.just_released(KeyCode::ArrowUp) {
        down.0 = false;
    }
}

fn detect_collisions(
    colliding_q: Query<(Entity, &CollidingEntities)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (_, colliding_entities) in colliding_q {
        if colliding_entities.len() > 0 {
            info!("T-Rex collision!");
            next_state.set(GameState::GameOver);
        }
    }
}

fn check_image_loaded(
    image: Res<ImageHandle>,
    mut rdr: MessageReader<AssetEvent<Image>>,
    mut next_state: ResMut<NextState<ImageState>>,
) {
    if let Some(handle) = &**image.as_ref() {
        for evt in rdr.read() {
            if evt.is_loaded_with_dependencies(handle) {
                next_state.set(ImageState::Loaded);
            }
        }
    }
}

fn update_collider(
    cfg: Res<Config>,
    image: Res<ImageHandle>,
    images: Res<Assets<Image>>,
    trex: Single<Entity, With<Trex>>,
    mut colliders: ResMut<ColliderMap>,
    mut commands: Commands,
) {
    if let Some(handle) = &**image {
        match images.get(handle) {
            Some(img) => {
                let rect = URect::new(
                    cfg.trex_sprite.offset.x,
                    cfg.trex_sprite.offset.y,
                    cfg.trex_sprite.offset.x + cfg.trex_sprite.size.x,
                    cfg.trex_sprite.offset.y + cfg.trex_sprite.size.y,
                );
                let collider = colliders.add(handle.id(), img, &rect, None);
                let mut entity = commands.entity(*trex);
                entity.insert(collider);
            }
            None => panic!(),
        }
    }
}
