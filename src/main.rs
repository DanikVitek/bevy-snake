use std::{env, time::Duration};

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, sprite::Anchor, utils::HashMap,
};

const CELL_SIZE: f32 = 32.0;
const FIELD_WIDTH: f32 = CELL_SIZE * 25.0;
const FIELD_HEIGHT: f32 = CELL_SIZE * 20.0;
const STEP_DURATION: Duration = Duration::from_millis(500);

fn main() {
    env::set_var("WGPU_BACKEND", "vulkan");
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Snake".to_string(),
                        resolution: (FIELD_WIDTH, FIELD_HEIGHT).into(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build(),
        )
        .add_systems(Startup, (setup, setup_snake))
        .add_event::<UpdateDirectionEvent>()
        .add_systems(
            Update,
            (
                move_snake,
                control_snake,
                update_tail_direction.after(move_snake),
                rotate_snake_sprite,
            ),
        )
        .run();
}

// #[derive(Debug, Clone, Copy, Default, States, PartialEq, Eq, Hash)]
// enum AppState {
//     #[default]
//     LoadHandles,
//     Setup,
// }

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
                next_move: Timer::new(STEP_DURATION, TimerMode::Repeating),
            },
            SpatialBundle::default(),
        ))
        .with_children(|snake| {
            let head_id = snake
                .spawn((
                    SnakeHead,
                    Direction::default(),
                    PrevDirection::default(),
                    make_sprite(asset_server.load("sprites/snake-head.png"), Vec2::ZERO),
                ))
                .id();
            snake.spawn((
                TailPiece,
                Direction::default(),
                PrevId(head_id),
                make_sprite(
                    asset_server.load("sprites/snake-end.png"),
                    Vec2 { x: 0.0, y: -1.0 },
                ),
            ));
        });
}

/// Make a sprite bundle at a given position, multiplied by `CELL_SIZE`
/// and the sprite anchor is set to [`Anchor::TopLeft`]
fn make_sprite(texture: Handle<Image>, _position @ Vec2 { x, y }: Vec2) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            anchor: Anchor::Center,
            ..Default::default()
        },
        visibility: Visibility::Visible,
        transform: Transform::from_xyz(x * CELL_SIZE, y * CELL_SIZE, 0.0),
        texture,
        ..Default::default()
    }
}

fn move_snake(
    time: Res<Time>,
    mut snake: Query<&mut Snake>,
    mut head: Query<(Entity, &Direction, &mut PrevDirection, &mut Transform)>,
    mut pieces: Query<(&Direction, &mut Transform), Without<PrevDirection>>,
    mut update_direction_ev: EventWriter<UpdateDirectionEvent>,
) {
    let mut snake = snake.get_single_mut().unwrap();
    if !snake.next_move.tick(time.delta()).just_finished() {
        return;
    }

    let (id, direction, mut prev_direction, mut transform) = head.get_single_mut().unwrap();
    let direction = *direction;
    translate(&mut transform, direction);
    if let PrevDirection(Some(prev_direction)) = &mut *prev_direction {
        if *prev_direction != direction {
            update_direction_ev.send(UpdateDirectionEvent(id));
            *prev_direction = direction;
        }
    } else {
        prev_direction.0 = Some(direction);
    }

    for (direction, mut transform) in pieces.iter_mut() {
        translate(&mut transform, *direction);
    }
}

fn translate(transform: &mut Transform, direction: Direction) {
    match direction {
        Direction::Left => {
            transform.translation.x -= CELL_SIZE;
            if transform.translation.x < -FIELD_WIDTH / 2.0 {
                transform.translation.x = FIELD_WIDTH / 2.0 - CELL_SIZE;
            }
        }
        Direction::Right => {
            transform.translation.x += CELL_SIZE;
            if transform.translation.x > FIELD_WIDTH / 2.0 {
                transform.translation.x = -FIELD_WIDTH / 2.0 + CELL_SIZE;
            }
        }
        Direction::Up => {
            transform.translation.y += CELL_SIZE;
            if transform.translation.y > FIELD_HEIGHT / 2.0 {
                transform.translation.y = -FIELD_HEIGHT / 2.0 + CELL_SIZE;
            }
        }
        Direction::Down => {
            transform.translation.y -= CELL_SIZE;
            if transform.translation.y < -FIELD_HEIGHT / 2.0 {
                transform.translation.y = FIELD_HEIGHT / 2.0 - CELL_SIZE;
            }
        }
    }
}

fn update_tail_direction(
    snake: Query<&Snake>,
    tail: Query<&PrevId>,
    mut directions: ParamSet<(
        Query<&Direction>,
        Query<(Entity, &PrevId, &mut Direction), With<TailPiece>>,
    )>,
    mut update_diection_ev: EventWriter<UpdateDirectionEvent>,
) {
    let snake = snake.get_single().unwrap();
    if !snake.next_move.just_finished() {
        return;
    }

    let prev_directions: HashMap<Entity, Direction> = tail
        .iter()
        .copied()
        .map(|PrevId(prev_id)| (prev_id, *directions.p0().get(prev_id).unwrap()))
        .collect();

    directions
        .p1()
        .for_each_mut(|(id, prev_id, mut direction)| {
            let prev_direction = prev_directions[&prev_id.0];
            if *direction != prev_direction {
                *direction = prev_direction;
                update_diection_ev.send(UpdateDirectionEvent(id))
            }
        });
}

fn control_snake(
    keyboard_input: Res<Input<KeyCode>>,
    mut snake_head: Query<&mut Direction, With<SnakeHead>>,
) {
    let mut direction = snake_head.get_single_mut().unwrap();
    for keycode in keyboard_input.get_just_pressed() {
        match keycode {
            KeyCode::Up if !direction.is_down() => *direction = Direction::Up,
            KeyCode::Down if !direction.is_up() => *direction = Direction::Down,
            KeyCode::Left if !direction.is_right() => *direction = Direction::Left,
            KeyCode::Right if !direction.is_left() => *direction = Direction::Right,
            _ => {}
        }
    }
}

fn rotate_snake_sprite(
    mut update_direction_ev: EventReader<UpdateDirectionEvent>,
    mut sprite: Query<(&mut Transform, &Direction)>,
) {
    for UpdateDirectionEvent(id) in update_direction_ev.iter().copied() {
        let (mut transform, direction) = sprite.get_mut(id).unwrap();
        transform.rotation = direction.into();
    }
}

#[derive(Component)]
struct Snake {
    next_move: Timer,
}

#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeTail;

#[derive(Component)]
struct TailPiece;

#[derive(Clone, Copy, Component)]
struct PrevId(Entity);

#[derive(Clone, Copy, Event)]
struct UpdateDirectionEvent(Entity);

#[derive(Clone, Copy, Default, Component, PartialEq, Eq)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Component)]
struct PrevDirection(Option<Direction>);

impl Direction {
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
}

impl From<Direction> for Quat {
    fn from(value: Direction) -> Self {
        Quat::from_rotation_z(match value {
            Direction::Up => 0.0,
            Direction::Down => std::f32::consts::PI,
            Direction::Left => std::f32::consts::FRAC_PI_2,
            Direction::Right => std::f32::consts::FRAC_PI_2 * 3.0,
        })
    }
}

impl From<&Direction> for Quat {
    #[inline(always)]
    fn from(value: &Direction) -> Self {
        Quat::from(*value)
    }
}
