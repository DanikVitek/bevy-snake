mod component;
mod event;
mod resource;
mod system;

use std::{env, time::Duration};

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, sprite::Anchor};

use crate::{
    component::{Direction, *},
    event::*,
    resource::*,
    system::*,
};

const CELL_SIZE: f32 = 32.0;
const FIELD_WIDTH: i32 = 24;
const FIELD_HEIGHT: i32 = 24;
const VISIBLE_FIELD_WIDTH: f32 = CELL_SIZE * FIELD_WIDTH as f32;
const VISIBLE_FIELD_HEIGHT: f32 = CELL_SIZE * FIELD_HEIGHT as f32;
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
                        resolution: (VISIBLE_FIELD_WIDTH, VISIBLE_FIELD_HEIGHT).into(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build(),
        )
        .init_resource::<ExcludeSnakeDistribution>()
        .add_systems(Startup, (setup, setup_snake))
        .add_event::<UpdateDirectionEvent>()
        .add_event::<StepEvent>()
        .add_event::<EatAppleEvent>()
        .add_systems(
            Update,
            (
                move_snake,
                control_snake,
                update_tail_direction.after(move_snake),
                rotate_snake_sprite,
                update_exclude_snake_distribution.after(move_snake),
                eat_apple.after(move_snake),
                move_eaten_apple.after(eat_apple),
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
                    SnakePiece,
                    Direction::default(),
                    PrevDirection::default(),
                    make_sprite(asset_server.load("sprites/snake-head.png"), IVec2::ZERO),
                ))
                .id();
            snake.spawn((
                SnakeTail,
                SnakePiece,
                Direction::default(),
                PrevId(head_id),
                make_sprite(
                    asset_server.load("sprites/snake-end.png"),
                    IVec2 { x: 0, y: -1 },
                ),
            ));
        });
    commands.spawn((
        Apple,
        make_sprite(asset_server.load("sprites/apple.png"), IVec2::ZERO),
    ));
}

/// Make a sprite bundle at a given position, multiplied by `CELL_SIZE`
fn make_sprite(texture: Handle<Image>, _position @ IVec2 { x, y }: IVec2) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            anchor: Anchor::Center,
            ..Default::default()
        },
        visibility: Visibility::Visible,
        transform: Transform::from_xyz(x as f32 * CELL_SIZE, y as f32 * CELL_SIZE, 0.0),
        texture,
        ..Default::default()
    }
}
