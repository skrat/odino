use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    assets::{ScoreSoundEffect, SpriteHandle},
    cacti::CACTI_SPAWN,
    config::{Config, D100, SPEED_TO_SCORE, UI_LAYER},
    dactyl::DACTYL_SPAWN,
    state::{GameState, Obstacle, ObstacleSystem, Score, Speed, not_in_state},
    util::{ColliderMap, add_layout},
};

pub struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(Speed(D100.init_speed))
            .insert_resource(Score::default())
            .insert_resource(HiScore::default())
            .insert_resource(ColliderMap::default())
            .insert_resource(ObstacleTimer(Timer::from_seconds(3.0, TimerMode::Once)))
            .insert_resource(ScoreTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
            .insert_resource(SpeedTimer(Timer::from_seconds(10., TimerMode::Repeating)))
            .init_resource::<NumLayout>()
            .add_systems(OnEnter(GameState::Paused), update_gamestate_label)
            .add_systems(OnEnter(GameState::Running), update_gamestate_label)
            .add_systems(OnEnter(GameState::GameOver), update_gamestate_label)
            .add_systems(OnEnter(GameState::Running), unpause_time)
            .add_systems(OnExit(GameState::Running), pause_time)
            .add_systems(OnEnter(GameState::GameOver), update_hiscore)
            .add_systems(OnExit(GameState::GameOver), reset_game)
            .add_systems(
                Startup,
                (
                    spawn_gamestate_label,
                    spawn_score,
                    spawn_hiscore,
                    pause_time,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    spawn_obstacles.run_if(in_state(GameState::Running)),
                    increment_score.run_if(in_state(GameState::Running)),
                    increment_speed.run_if(in_state(GameState::Running)),
                ),
            )
            .add_systems(
                Update,
                (
                    quit_on_q,
                    play_on_space.run_if(not_in_state(GameState::Running)),
                    pause_on_esc
                        .run_if(in_state(GameState::Running).or_else(in_state(GameState::Paused))),
                ),
            );

        app.add_systems(
            Update,
            bevy::dev_tools::states::log_transitions::<GameState>,
        );
    }
}

#[derive(Resource, Deref)]
struct NumLayout(Handle<TextureAtlasLayout>);

#[derive(Resource, Deref, DerefMut)]
struct ObstacleTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct ScoreTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct SpeedTimer(Timer);

#[derive(Resource, Default)]
struct HiScore(f32);

#[derive(Component)]
struct GameStateLabel;

#[derive(Component, Clone)]
struct ScoreCounterNum;

#[derive(Component, Clone)]
struct HiScoreCounterNum;

impl FromWorld for NumLayout {
    fn from_world(world: &mut World) -> Self {
        NumLayout(add_layout(world, |cfg| cfg.num_sprite.into()))
    }
}

fn spawn_obstacles(
    t: Res<Time<Fixed>>,
    score: Res<Score>,
    mut timer: ResMut<ObstacleTimer>,
    cacti_spawn: Res<ObstacleSystem<CACTI_SPAWN>>,
    dactyl_spawn: Res<ObstacleSystem<DACTYL_SPAWN>>,
    mut commands: Commands,
) {
    timer.tick(t.delta());
    if timer.just_finished() {
        if score.0 > 200. && rand::random_range(0..=10) > 8 {
            commands.run_system(dactyl_spawn.0);
            timer.set_duration(Duration::from_secs(rand::random_range(2..=3) as u64));
        } else {
            commands.run_system(cacti_spawn.0);
            timer.set_duration(Duration::from_secs(rand::random_range(1..3) as u64));
        }
        timer.reset();
    }
}

fn gamestate_label_str(s: &GameState) -> &'static str {
    match s {
        GameState::Init => "<Space> to play",
        GameState::Paused => "PAUSED | <Space> to play",
        GameState::Running => "",
        GameState::GameOver => "GAME OVER | <Space> to play",
    }
}

fn spawn_gamestate_label(cfg: Res<Config>, state: Res<State<GameState>>, mut commands: Commands) {
    commands.spawn((
        Text2d::new(gamestate_label_str(state.get())),
        TextFont::from_font_size(FontSize::Px(16.0)),
        TextColor::BLACK,
        Transform::from_xyz(cfg.size.x * 0.5, cfg.size.y * 0.5, UI_LAYER),
        GameStateLabel,
    ));
}

fn update_gamestate_label(
    state: Res<State<GameState>>,
    q: Single<&mut Text2d, With<GameStateLabel>>,
) {
    let mut t = q.into_inner();
    *t = Text2d::new(gamestate_label_str(state.get()));
}

fn spawn_score(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout: Res<NumLayout>,
    commands: Commands,
) {
    spawn_counter(
        cfg.size.x * 0.8,
        ScoreCounterNum,
        cfg,
        image,
        layout,
        commands,
    );
}
fn spawn_hiscore(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout: Res<NumLayout>,
    commands: Commands,
) {
    spawn_counter(
        cfg.size.x * 0.65,
        HiScoreCounterNum,
        cfg,
        image,
        layout,
        commands,
    );
}

fn spawn_counter<T: Component + Clone>(
    x: f32,
    marker: T,
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout: Res<NumLayout>,
    mut commands: Commands,
) {
    commands
        .spawn((
            Visibility::default(),
            Transform::from_xyz(x, cfg.size.y * 0.75, UI_LAYER),
        ))
        .with_children(|parent| {
            for i in 0..5 {
                parent.spawn((
                    Sprite::from_atlas_image(
                        image.clone(),
                        TextureAtlas {
                            layout: layout.clone(),
                            index: 0,
                        },
                    ),
                    Transform::from_xyz((i * (cfg.num_sprite.size.x as usize + 1)) as f32, 0., 0.),
                    marker.clone(),
                ));
            }
        });
}

fn increment_score(
    t: Res<Time<Fixed>>,
    speed: Res<Speed>,
    mut score: ResMut<Score>,
    mut timer: ResMut<ScoreTimer>,
    sound_effect: Res<ScoreSoundEffect>,
    digits: Query<&mut Sprite, With<ScoreCounterNum>>,
    mut commands: Commands,
) {
    timer.tick(t.delta());
    if timer.just_finished() {
        score.0 = score.0 + (speed.0 * SPEED_TO_SCORE);
        set_counter(score.0.ceil() as u32, digits);
        if score.0.ceil() as u32 % 100 == 0 {
            commands.spawn((
                AudioPlayer::new(sound_effect.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn increment_speed(t: Res<Time<Fixed>>, mut speed: ResMut<Speed>, mut timer: ResMut<SpeedTimer>) {
    timer.tick(t.delta());
    if timer.just_finished() {
        speed.0 += 30.;
    }
}

fn quit_on_q(input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}

fn play_on_space(
    mut t: ResMut<Time<Virtual>>,
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Running);
        t.unpause();
    }
}

fn pause_on_esc(
    input: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match game_state.get() {
            GameState::Running => {
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                next_state.set(GameState::Running);
            }
            _ => (),
        }
    }
}

fn pause_time(mut virt: ResMut<Time<Virtual>>, mut phys: ResMut<Time<Physics>>) {
    virt.pause();
    phys.pause();
}

fn unpause_time(mut virt: ResMut<Time<Virtual>>, mut phys: ResMut<Time<Physics>>) {
    virt.unpause();
    phys.unpause();
}

fn reset_game(
    cfg: Res<Config>,
    mut speed: ResMut<Speed>,
    mut score: ResMut<Score>,
    mut commands: Commands,
) {
    speed.0 = cfg.init_speed;
    score.0 = 0.;
    commands.despawn_all::<With<Obstacle>>();
}

fn update_hiscore(
    score: Res<Score>,
    mut hiscore: ResMut<HiScore>,
    query: Query<&mut Sprite, With<HiScoreCounterNum>>,
) {
    hiscore.0 = score.0.max(hiscore.0);
    set_counter(hiscore.0.ceil() as u32, query);
}

fn set_counter<T: Component>(val: u32, query: Query<&mut Sprite, With<T>>) {
    let chars = format!("{:0>5}", val).chars().collect::<Vec<_>>();

    let mut i = 0;
    for mut sprite in query {
        let d = chars[i].to_string().parse::<usize>().unwrap();
        let atlas = sprite.texture_atlas.as_mut().unwrap();
        atlas.index = d;
        i += 1;
    }
}
