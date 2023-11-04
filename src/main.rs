mod component;
mod event;
mod resource;
mod system;

use std::{env, time::Duration};

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, sprite::Anchor};
use cfg_if::cfg_if;

use crate::{component::*, event::*, resource::*, system::*};

const SPRITE_SIZE: u8 = 8;
const CELL_SIZE: f32 = SPRITE_SIZE as f32 * 4.0;
const FIELD_WIDTH: i32 = 24;
const FIELD_HEIGHT: i32 = 24;
const VISIBLE_FIELD_WIDTH: f32 = CELL_SIZE * FIELD_WIDTH as f32;
const VISIBLE_FIELD_HEIGHT: f32 = CELL_SIZE * FIELD_HEIGHT as f32;
const STEP_DURATION: Duration = Duration::from_millis(500);

fn main() {
    env::set_var("WGPU_BACKEND", {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                "webgpu"
            // } else if #[cfg(target_os = "windows")] {
            //     "dx12"
            } else if #[cfg(target_os = "macos")] {
                "metal"
            } else {
                "vulkan"
            }
        }
    }); // TODO: replace with `vulkan` when it's fixed
    App::new()
        .add_plugins((
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
            SnakeGame,
        ))
        .run();
}

struct SnakeGame;

impl Plugin for SnakeGame {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExcludeSnakeDistribution>()
            .add_state::<AppState>()
            .init_resource::<Sprites>()
            .add_systems(OnEnter(AppState::LoadHandles), load_sprites)
            .add_systems(
                Update,
                check_textures.run_if(in_state(AppState::LoadHandles)),
            )
            .add_systems(OnEnter(AppState::Setup), setup)
            .add_event::<UpdateDirectionEvent>()
            .add_event::<StepEvent>()
            .add_event::<EatAppleEvent>()
            .init_resource::<Events<ElongateEvent>>()
            // NOTE: ^ must be persisted between frames to account for the timer
            .add_event::<WinEvent>()
            .add_systems(
                Update,
                control_snake
                    .before(move_snake)
                    .run_if(in_state(AppState::Ready)),
            )
            .insert_resource(Time::<Fixed>::from_duration(STEP_DURATION))
            .add_systems(
                FixedUpdate,
                (
                    (move_snake, update_tail_direction, rotate_snake_sprite).chain(),
                    update_exclude_snake_distribution.after(move_snake),
                    eat_apple.after(move_snake),
                    move_eaten_apple.after(eat_apple),
                    elongate_body.after(eat_apple),
                )
                    .run_if(in_state(AppState::Ready)),
            );
    }
}

#[derive(Debug, Clone, Copy, Default, States, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    LoadHandles,
    Setup,
    Ready,
}

fn load_sprites(mut sprites: ResMut<Sprites>, asset_server: Res<AssetServer>) {
    sprites.load(&asset_server);
}

fn check_textures(
    mut state: ResMut<NextState<AppState>>,
    sprites: Res<Sprites>,
    asset_server: Res<AssetServer>,
) {
    if sprites.check_loaded(&asset_server) {
        state.set(AppState::Setup);
    }
}

fn setup(mut commands: Commands, sprites: Res<Sprites>, mut state: ResMut<NextState<AppState>>) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::DARK_GREEN),
        },
        ..Default::default()
    });
    commands
        .spawn((Snake, SpatialBundle::default()))
        .with_children(|snake| {
            let head_id = snake
                .spawn((
                    SnakeHead,
                    SnakePiece,
                    MoveDirection::default(),
                    PrevDirection::default(),
                    make_sprite(sprites.head(), Vec3::ZERO, None::<Quat>),
                ))
                .id();
            snake.spawn((
                SnakeTail::End,
                SnakePiece,
                MoveDirection::default(),
                PrevId(head_id),
                make_sprite(sprites.tail_end(), IVec3::new(0, -1, -1), None::<Quat>),
            ));
        });
    commands.spawn((
        Apple,
        make_sprite(sprites.apple(), Vec3::new(0.0, 0.0, -2.0), None::<Quat>),
    ));

    state.set(AppState::Ready);
}

/// Make a sprite bundle at a given position, multiplied by `CELL_SIZE`
fn make_sprite(
    texture: Handle<Image>,
    position: impl IntoSnakePosition,
    direction: Option<impl Into<Quat>>,
) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            anchor: Anchor::Center,
            ..Default::default()
        },
        visibility: Visibility::Visible,
        transform: {
            Transform::from_translation(position.into_snake_position())
                .with_rotation(direction.map(Into::into).unwrap_or_default())
        },
        texture,
        ..Default::default()
    }
}

trait IntoSnakePosition {
    fn into_snake_position(self) -> Vec3;
}

impl IntoSnakePosition for IVec2 {
    #[inline]
    fn into_snake_position(self) -> Vec3 {
        Vec3::new(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE, 0.0)
    }
}

impl IntoSnakePosition for IVec3 {
    #[inline]
    fn into_snake_position(self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CELL_SIZE,
            self.y as f32 * CELL_SIZE,
            self.z as f32,
        )
    }
}

impl IntoSnakePosition for Vec2 {
    #[inline(always)]
    fn into_snake_position(self) -> Vec3 {
        Vec3::new(self.x, self.y, 0.0)
    }
}

impl IntoSnakePosition for Vec3 {
    #[inline(always)]
    fn into_snake_position(self) -> Vec3 {
        self
    }
}
