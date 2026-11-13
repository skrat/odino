use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{assets::*, config::*, state::*, util::*};

pub struct TrexPlugin;

impl Plugin for TrexPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunningLayout>()
            .init_resource::<DuckingLayout>()
            .insert_resource(JumpKeyDown(false))
            .add_systems(Startup, spawn_trex)
            .add_systems(OnEnter(SpriteState::Loaded), update_collider)
            .add_systems(OnExit(GameState::GameOver), reset_trex)
            .add_systems(
                FixedUpdate,
                (
                    animate_trex.run_if(in_state(GameState::Running)),
                    jump_power_up.run_if(in_state(GameState::Running)),
                    jumping_trex.run_if(
                        in_state(GameState::Running).and_then(trex_in_state(TrexState::Jumping)),
                    ),
                    detect_collisions.run_if(in_state(GameState::Running)),
                ),
            )
            .add_systems(
                Update,
                (
                    jump_on_key.run_if(in_state(GameState::Running)),
                    duck_on_key.run_if(in_state(GameState::Running)),
                ),
            );
    }
}

pub fn trex_in_state(state: TrexState) -> impl FnMut(Option<Single<&Trex>>) -> bool {
    move |trex: Option<Single<&Trex>>| match trex {
        Some(trex) => trex.state == state,
        None => false,
    }
}

/// Resources

#[derive(Resource, Default)]
struct JumpKeyDown(bool);

#[derive(Resource, Deref)]
struct RunningLayout(Handle<TextureAtlasLayout>);

impl FromWorld for RunningLayout {
    fn from_world(world: &mut World) -> Self {
        RunningLayout(add_layout(world, |cfg| cfg.trex_sprite.into()))
    }
}

#[derive(Resource, Deref)]
struct DuckingLayout(Handle<TextureAtlasLayout>);

impl FromWorld for DuckingLayout {
    fn from_world(world: &mut World) -> Self {
        DuckingLayout(add_layout(world, |cfg| cfg.trex_ducking_sprite.into()))
    }
}

/// Components

#[derive(Component)]
struct TrexWrapper;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum TrexState {
    #[default]
    Running,
    Jumping,
    Ducking,
}

#[derive(Component, Default)]
pub struct Trex {
    pub state: TrexState,
    pub jumped_at: f64,
    pub jump_power: f32,
}

#[derive(Component, Deref, DerefMut)]
struct TrexAnimationTimer(Timer);

/// Systems

fn spawn_trex(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout: Res<RunningLayout>,
    colliders: Res<ColliderMap>,
    mut commands: Commands,
) {
    commands.spawn((
        Visibility::default(),
        Transform::from_translation(trex_running_pos(&cfg)),
        TrexWrapper,
        children![(
            Sprite::from_atlas_image(
                image.clone(),
                TextureAtlas {
                    layout: layout.clone(),
                    index: cfg.trex_running_start,
                },
            ),
            TrexAnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Transform::from_translation(Vec3::ZERO),
            TranslationInterpolation,
            RigidBody::Kinematic,
            colliders.get_collider(&cfg.trex_sprite.rect(0)),
            CollidingEntities::default(),
            Trex::default(),
        )],
    ));
}

fn animate_trex(
    t: Res<Time<Fixed>>,
    cfg: Res<Config>,
    query: Single<(&Trex, &mut TrexAnimationTimer, &mut Sprite)>,
) {
    let (trex, mut timer, mut sprite) = query.into_inner();
    timer.tick(t.delta());

    if trex.state != TrexState::Jumping
        && timer.just_finished()
        && let Some(atlas) = &mut sprite.texture_atlas
    {
        let start = match trex.state {
            TrexState::Running => cfg.trex_running_start,
            TrexState::Ducking => cfg.trex_ducking_start,
            _ => unreachable!(),
        };
        let end = match trex.state {
            TrexState::Running => cfg.trex_running_end,
            TrexState::Ducking => cfg.trex_ducking_end,
            _ => unreachable!(),
        };
        atlas.index = if atlas.index == end {
            start
        } else {
            atlas.index + 1
        };
    }
}

fn jumping_trex(
    t: Res<Time<Fixed>>,
    cfg: Res<Config>,
    down: Res<JumpKeyDown>,
    q: Single<(&mut Trex, &mut Transform, &mut Sprite)>,
) {
    let (mut trex, mut tf, mut sprite) = q.into_inner();
    let delta = t.elapsed_secs_f64() - trex.jumped_at;
    const SPEED: f32 = 1.5;
    let d = trex.jump_power * (1. / SPEED);
    let p = 1.0 - (0.5 - delta as f32 / d).abs() * 2.0;
    let e = EaseFunction::QuadraticOut.sample(p).unwrap_or(0.);
    tf.translation.y = cfg.size.y * 0.75 * trex.jump_power * e;
    if delta >= d as f64 {
        if down.0 {
            // Re-jump
            trex.jumped_at = t.elapsed_secs_f64();
            trex.jump_power = cfg.init_jump_power;
        } else {
            // Finish jump
            trex.state = TrexState::Running;
            let atlas = sprite.texture_atlas.as_mut().unwrap();
            atlas.index = cfg.trex_running_start;
        }
    }
}

fn jump_power_up(t: Res<Time<Fixed>>, down: Res<JumpKeyDown>, query: Single<&mut Trex>) {
    let mut trex = query.into_inner();
    if down.0 {
        trex.jump_power = (trex.jump_power + t.delta().as_secs_f32() * 2.2).min(0.75);
    }
}

fn jump_on_key(
    t: Res<Time<Fixed>>,
    cfg: Res<Config>,
    input: Res<ButtonInput<KeyCode>>,
    sound_effect: Res<KeySoundEffect>,
    mut down: ResMut<JumpKeyDown>,
    query: Single<(&mut Trex, &mut Sprite)>,
    mut commands: Commands,
) {
    const KEY0: KeyCode = KeyCode::Space;
    const KEY1: KeyCode = KeyCode::Space;
    let (mut trex, mut sprite) = query.into_inner();
    if input.just_pressed(KEY0) || input.just_pressed(KEY1) {
        match trex.state {
            TrexState::Running => {
                let atlas = sprite.texture_atlas.as_mut().unwrap();
                trex.state = TrexState::Jumping;
                trex.jumped_at = t.elapsed_secs_f64();
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
    if input.just_released(KEY0) || input.just_released(KEY1) {
        down.0 = false;
    }
}

fn duck_on_key(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout_ducking: Res<DuckingLayout>,
    layout_running: Res<RunningLayout>,
    colliders: Res<ColliderMap>,
    input: Res<ButtonInput<KeyCode>>,
    trex: Single<(&mut Trex, &mut Sprite, &mut Collider)>,
    wrapper: Single<&mut Transform, With<TrexWrapper>>,
) {
    const DUCKING_KEY: KeyCode = KeyCode::ArrowDown;
    let (mut trex, mut sprite, mut collider) = trex.into_inner();
    let mut wrapper_pos = wrapper.into_inner();
    if input.just_pressed(DUCKING_KEY) && trex.state == TrexState::Running {
        trex.state = TrexState::Ducking;
        *sprite = Sprite::from_atlas_image(
            image.clone(),
            TextureAtlas {
                layout: layout_ducking.clone(),
                index: cfg.trex_ducking_start,
            },
        );
        *collider = colliders
            .get(&cfg.trex_ducking_sprite.rect(0))
            .unwrap()
            .clone();
        *wrapper_pos = Transform::from_translation(trex_ducking_pos(&cfg));
    }
    if input.just_released(DUCKING_KEY) && trex.state == TrexState::Ducking {
        trex.state = TrexState::Running;
        *sprite = Sprite::from_atlas_image(
            image.clone(),
            TextureAtlas {
                layout: layout_running.clone(),
                index: cfg.trex_running_start,
            },
        );
        *collider = colliders.get(&cfg.trex_sprite.rect(0)).unwrap().clone();
        *wrapper_pos = Transform::from_translation(trex_running_pos(&cfg));
    }
}

fn detect_collisions(
    hit_sound: Res<HitSoundEffect>,
    colliding_q: Query<(Entity, &CollidingEntities)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (_, colliding_entities) in colliding_q {
        if colliding_entities.len() > 0 {
            info!("T-Rex collision!");
            next_state.set(GameState::GameOver);
            commands.spawn((
                AudioPlayer::new(hit_sound.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn update_collider(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    images: Res<Assets<Image>>,
    trex: Single<(Entity, &Trex)>,
    mut colliders: ResMut<ColliderMap>,
    mut commands: Commands,
) {
    match images.get(image.id()) {
        Some(img) => {
            let running_rect: URect = cfg.trex_sprite.rect(0);
            let running_collider = colliders.add(img, &running_rect, None);

            let ducking_rect: URect = cfg.trex_ducking_sprite.rect(0);
            let ducking_collider = colliders.add(img, &ducking_rect, None);

            let (ent, trex) = trex.into_inner();
            let mut entity = commands.entity(ent);

            match trex.state {
                TrexState::Ducking => entity.insert(ducking_collider),
                _ => entity.insert(running_collider),
            };
        }
        None => panic!(),
    }
}

fn reset_trex(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout: Res<RunningLayout>,
    colliders: Res<ColliderMap>,
    wrapper: Single<Entity, With<TrexWrapper>>,
    mut commands: Commands,
) {
    commands.entity(wrapper.into_inner()).despawn();
    spawn_trex(cfg, image, layout, colliders, commands);
}

fn trex_running_pos(cfg: &Config) -> Vec3 {
    Vec3::new(50., cfg.trex_sprite.size.y as f32 * 0.35, TREX_LAYER)
}

fn trex_ducking_pos(cfg: &Config) -> Vec3 {
    Vec3::new(50., cfg.trex_sprite.size.y as f32 * 0.2, TREX_LAYER)
}
