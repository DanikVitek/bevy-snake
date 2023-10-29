use std::{env, time::Duration};

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, sprite::Anchor};

const CELL_SIZE: f32 = 32.0;

fn main() {
    env::set_var("WGPU_BACKEND", "vulkan");
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Snake".to_string(),
                        resolution: (800.0, 640.0).into(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build(),
        )
        .add_systems(Startup, (setup, setup_snake))
        .add_systems(Update, (move_snake, control_snake))
        .run();
}

#[derive(Debug, Clone, Copy, Default, States, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    LoadHandles,
    Setup,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::DARK_GREEN),
        },
        ..Default::default()
    });
}

fn setup_snake(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Snake {
                next_move: Timer::new(Duration::from_millis(1000), TimerMode::Repeating),
            },
            SpatialBundle::default(),
        ))
        .with_children(|snake| {
            snake.spawn((
                SnakeHead {
                    direction: Direction::default(),
                },
                make_sprite(asset_server.load("sprites/snake-head.png"), Vec2::ZERO),
            ));
            snake
                .spawn((SnakeTail, SpatialBundle::default()))
                .with_children(|tail| {
                    tail.spawn((
                        TailPiece {
                            direction: Direction::default(),
                        },
                        make_sprite(
                            asset_server.load("sprites/snake-end.png"),
                            Vec2 { x: 0.0, y: -1.0 },
                        ),
                    ));
                });
        });
}

/// Make a sprite bundle at a given position, multiplied by `32.0`
/// and the sprite anchor is set to [`Anchor::TopLeft`]
fn make_sprite(texture: Handle<Image>, _position @ Vec2 { x, y }: Vec2) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(32.0, 32.0)),
            anchor: Anchor::TopLeft,
            ..Default::default()
        },
        visibility: Visibility::Visible,
        transform: Transform::from_xyz(x * 32.0, y * 32.0, 0.0),
        texture,
        ..Default::default()
    }
}

fn move_snake(
    time: Res<Time>,
    mut snake: Query<&mut Snake>,
    mut snake_body: ParamSet<(
        Query<(&SnakeHead, &mut Transform)>,
        Query<(&mut TailPiece, &mut Transform)>,
    )>,
) {
    let mut snake = snake.get_single_mut().unwrap();
    if snake.next_move.tick(time.delta()).just_finished() {
        let mut p0 = snake_body.p0();
        let (head, mut transform) = p0.get_single_mut().unwrap();
        match head.direction {
            Direction::Left => transform.translation.x -= CELL_SIZE,
            Direction::Right => transform.translation.x += CELL_SIZE,
            Direction::Up => transform.translation.y += CELL_SIZE,
            Direction::Down => transform.translation.y -= CELL_SIZE,
        }
        drop(p0);
        for (mut tail, mut transform) in snake_body.p1().iter_mut() {
            match tail.direction {
                Direction::Left => transform.translation.x -= CELL_SIZE,
                Direction::Right => transform.translation.x += CELL_SIZE,
                Direction::Up => transform.translation.y += CELL_SIZE,
                Direction::Down => transform.translation.y -= CELL_SIZE,
            }
        }
    }
}

fn control_snake(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Snake>) {
    // for mut snake in query.iter_mut() {
    //     if keyboard_input.just_pressed(KeyCode::Left) && !snake.direction.is_right() {
    //         snake.direction = Direction::Left;
    //     } else if keyboard_input.just_pressed(KeyCode::Right) && !snake.direction.is_left() {
    //         snake.direction = Direction::Right;
    //     } else if keyboard_input.just_pressed(KeyCode::Up) && !snake.direction.is_down() {
    //         snake.direction = Direction::Up;
    //     } else if keyboard_input.just_pressed(KeyCode::Down) && !snake.direction.is_up() {
    //         snake.direction = Direction::Down;
    //     }
    // }
}

#[derive(Component)]
struct Snake {
    next_move: Timer,
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct SnakeTail;

#[derive(Component)]
struct TailPiece {
    direction: Direction,
}

#[derive(Default)]
pub enum Direction {
    Left,
    Right,
    #[default]
    Up,
    Down,
}

impl Direction {
    /// Returns `true` if the direction is [`Left`].
    ///
    /// [`Left`]: Direction::Left
    #[must_use]
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }

    /// Returns `true` if the direction is [`Right`].
    ///
    /// [`Right`]: Direction::Right
    #[must_use]
    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right)
    }

    /// Returns `true` if the direction is [`Up`].
    ///
    /// [`Up`]: Direction::Up
    #[must_use]
    pub fn is_up(&self) -> bool {
        matches!(self, Self::Up)
    }

    /// Returns `true` if the direction is [`Down`].
    ///
    /// [`Down`]: Direction::Down
    #[must_use]
    pub fn is_down(&self) -> bool {
        matches!(self, Self::Down)
    }
}
