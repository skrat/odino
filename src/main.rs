use bevy::{camera::ScalingMode, prelude::*};

mod constants;
mod debug;
mod ground;
mod state;
mod trex;

use crate::{constants::*, debug::*, ground::*, state::*, trex::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(D100)
        .add_systems(Startup, (setup_camera, spawn_background))
        .add_plugins((GameControlPlugin, TrexPlugin, GroundPlugin, DebugPlugin))
        .init_state::<GameState>();

    #[cfg(feature = "dev")]
    app.add_systems(
        Update,
        bevy::dev_tools::states::log_transitions::<GameState>,
    );

    app.run();
}

#[derive(Component)]
struct GameStateLabel;

#[derive(Component)]
struct ScoreCounter;

#[derive(Component)]
struct ScoreCounterNum;

#[derive(Component, Deref, DerefMut)]
struct ScoreTimer(Timer);

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

struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Speed(D100.init_speed))
            .insert_resource(Score(0.))
            .add_systems(Startup, (spawn_gamestate_label, spawn_counter))
            .add_systems(OnEnter(GameState::Paused), update_gamestate_label)
            .add_systems(OnEnter(GameState::Running), update_gamestate_label)
            .add_systems(Update, increment_score.run_if(in_state(GameState::Running)))
            .add_systems(
                Update,
                (
                    quit_on_q,
                    play_on_space.run_if(not_in_state(GameState::Running)),
                    pause_on_esc
                        .run_if(in_state(GameState::Running).or_else(in_state(GameState::Paused))),
                ),
            );
    }
}

fn gamestate_label_str(s: &GameState) -> &'static str {
    match s {
        GameState::Init => "<Space> to play",
        GameState::Paused => "PAUSED | <Space> to play",
        GameState::Running => "",
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
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let tex = asset_server.load(SPRITE_PATH);
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
                        tex.clone(),
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

fn increment_score(
    t: Res<Time<Virtual>>,
    speed: Res<Speed>,
    mut score: ResMut<Score>,
    timer: Single<&mut ScoreTimer>,
    digits: Query<&mut Sprite, With<ScoreCounterNum>>,
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
    mut t: ResMut<Time<Virtual>>,
    input: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match game_state.get() {
            GameState::Running => {
                next_state.set(GameState::Paused);
                t.pause();
            }
            GameState::Paused => {
                next_state.set(GameState::Running);
                t.unpause();
            }
            _ => (),
        }
    }
}
