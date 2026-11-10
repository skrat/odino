use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    assets::{ScoreSoundEffect, SpriteHandle},
    config::*,
    state::*,
    util::ColliderMap,
};

pub struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(Speed(D100.init_speed))
            .insert_resource(Score::default())
            .insert_resource(ColliderMap::default())
            .add_systems(
                Startup,
                (
                    spawn_gamestate_label,
                    spawn_counter,
                    spawn_speed_timer,
                    pause_time,
                ),
            )
            .add_systems(OnEnter(GameState::Paused), update_gamestate_label)
            .add_systems(OnEnter(GameState::Running), update_gamestate_label)
            .add_systems(OnEnter(GameState::GameOver), update_gamestate_label)
            .add_systems(OnEnter(GameState::Running), unpause_time)
            .add_systems(OnExit(GameState::Running), pause_time)
            .add_systems(OnExit(GameState::GameOver), reset_game)
            .add_systems(
                FixedUpdate,
                (
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

        #[cfg(feature = "dev")]
        app.add_systems(
            Update,
            bevy::dev_tools::states::log_transitions::<GameState>,
        );
    }
}

#[derive(Component)]
struct GameStateLabel;

#[derive(Component)]
struct ScoreCounter;

#[derive(Component)]
struct ScoreCounterNum;

#[derive(Component, Deref, DerefMut)]
struct ScoreTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct SpeedTimer(Timer);

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
        Transform::from_xyz(cfg.size.x * 0.5, cfg.size.y * 0.5, 0.),
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

fn spawn_counter(
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let layout = atlas_layouts.add(cfg.num_sprite);
    commands
        .spawn((
            Transform::from_xyz(cfg.size.x * 0.8, cfg.size.y * 0.8, 100.),
            Visibility::default(),
            ScoreTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ScoreCounter,
        ))
        .with_children(|parent| {
            for i in 0..5 {
                parent.spawn((
                    Sprite::from_atlas_image(
                        image.clone(),
                        TextureAtlas {
                            layout: layout.clone(),
                            index: i,
                        },
                    ),
                    Transform::from_xyz((i * (cfg.num_sprite.size.x as usize + 1)) as f32, 0., 0.),
                    ScoreCounterNum,
                ));
            }
        });
}

fn spawn_speed_timer(mut commands: Commands) {
    commands.spawn(SpeedTimer(Timer::from_seconds(10., TimerMode::Repeating)));
}

fn increment_score(
    t: Res<Time<Fixed>>,
    speed: Res<Speed>,
    sound_effect: Res<ScoreSoundEffect>,
    mut score: ResMut<Score>,
    timer: Single<&mut ScoreTimer>,
    digits: Query<&mut Sprite, With<ScoreCounterNum>>,
    mut commands: Commands,
) {
    let mut timer = timer.into_inner();
    timer.tick(t.delta());

    if timer.just_finished() {
        let val = score.0 + (speed.0 * SPEED_TO_SCORE);
        *score = Score(val);
        let chars = format!("{:0>5}", val.ceil() as u32)
            .chars()
            .collect::<Vec<_>>();

        let mut i = 0;
        for mut sprite in digits {
            let d = chars[i].to_string().parse::<usize>().unwrap();
            let atlas = sprite.texture_atlas.as_mut().unwrap();
            atlas.index = d;
            i += 1;
        }
        if val.ceil() as u32 % 100 == 0 {
            commands.spawn((
                AudioPlayer::new(sound_effect.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn increment_speed(t: Res<Time<Fixed>>, mut speed: ResMut<Speed>, timer: Single<&mut SpeedTimer>) {
    let mut timer = timer.into_inner();
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
