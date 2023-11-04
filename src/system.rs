use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::Rng;

use crate::component::*;
use crate::resource::{ExcludeSnakeDistribution, Sprites};
use crate::{event::*, make_sprite, IntoSnakePosition};
use crate::{CELL_SIZE, VISIBLE_FIELD_HEIGHT, VISIBLE_FIELD_WIDTH};

pub fn move_snake(
    mut head: Query<(Entity, &MoveDirection, &mut PrevDirection, &mut Transform), With<SnakeHead>>,
    mut pieces: Query<(&MoveDirection, &mut Transform), (Without<PrevDirection>, With<SnakeTail>)>,
    mut update_direction_ev: EventWriter<UpdateDirectionEvent>,
    mut step_ev: EventWriter<StepEvent>,
    mut elongate_ev: ResMut<Events<ElongateEvent>>,
) {
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

    if elongate_ev.is_empty() {
        pieces.for_each_mut(|(direction, mut transform)| translate(&mut transform, *direction));
    } else {
        elongate_ev.clear();
    }

    step_ev.send(StepEvent);
}

fn translate(transform: &mut Transform, direction: MoveDirection) {
    match direction {
        MoveDirection::Left => {
            transform.translation.x -= CELL_SIZE;
            if transform.translation.x < -VISIBLE_FIELD_WIDTH / 2.0 {
                transform.translation.x = VISIBLE_FIELD_WIDTH / 2.0 - CELL_SIZE;
            }
        }
        MoveDirection::Right => {
            transform.translation.x += CELL_SIZE;
            if transform.translation.x > VISIBLE_FIELD_WIDTH / 2.0 {
                transform.translation.x = -VISIBLE_FIELD_WIDTH / 2.0 + CELL_SIZE;
            }
        }
        MoveDirection::Up => {
            transform.translation.y += CELL_SIZE;
            if transform.translation.y > VISIBLE_FIELD_HEIGHT / 2.0 {
                transform.translation.y = -VISIBLE_FIELD_HEIGHT / 2.0 + CELL_SIZE;
            }
        }
        MoveDirection::Down => {
            transform.translation.y -= CELL_SIZE;
            if transform.translation.y < -VISIBLE_FIELD_HEIGHT / 2.0 {
                transform.translation.y = VISIBLE_FIELD_HEIGHT / 2.0 - CELL_SIZE;
            }
        }
    }
}

pub fn update_exclude_snake_distribution(
    mut step_ev: EventReader<StepEvent>,
    snake: Query<&Transform, With<SnakePiece>>,
    mut exclude_snake_distribution: ResMut<ExcludeSnakeDistribution>,
) {
    for _ in step_ev.read() {
        exclude_snake_distribution.positions = snake
            .iter()
            .map(|transform| {
                let x = (transform.translation.x / CELL_SIZE).round() as i32;
                let y = (transform.translation.y / CELL_SIZE).round() as i32;
                IVec2 { x, y }
            })
            .collect();
    }
}

pub fn update_tail_direction(
    tail: Query<&PrevId>,
    mut directions: ParamSet<(
        Query<&MoveDirection>,
        Query<(Entity, &PrevId, &mut MoveDirection), With<SnakeTail>>,
    )>,
    mut update_diection_ev: EventWriter<UpdateDirectionEvent>,
) {
    let prev_directions: HashMap<Entity, MoveDirection> = tail
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

pub fn control_snake(
    keyboard_input: Res<Input<KeyCode>>,
    mut snake_head: Query<&mut MoveDirection, With<SnakeHead>>,
) {
    let mut direction = snake_head.get_single_mut().unwrap();
    for keycode in keyboard_input.get_just_pressed() {
        match keycode {
            KeyCode::Up if !direction.is_down() => *direction = MoveDirection::Up,
            KeyCode::Down if !direction.is_up() => *direction = MoveDirection::Down,
            KeyCode::Left if !direction.is_right() => *direction = MoveDirection::Left,
            KeyCode::Right if !direction.is_left() => *direction = MoveDirection::Right,
            _ => {}
        }
    }
}

pub fn rotate_snake_sprite(
    mut update_direction_ev: EventReader<UpdateDirectionEvent>,
    mut sprite: Query<(&mut Transform, &MoveDirection)>,
) {
    for UpdateDirectionEvent(id) in update_direction_ev.read().copied() {
        let (mut transform, direction) = sprite.get_mut(id).unwrap();
        transform.rotation = direction.into();
    }
}

pub fn eat_apple(
    mut step_ev: EventReader<StepEvent>,
    snake_head: Query<&Transform, With<SnakeHead>>,
    apple: Query<(Entity, &Transform), With<Apple>>,
    mut eat_apple_ev: EventWriter<EatAppleEvent>,
) {
    step_ev.read().for_each(|_| {
        let head_transform = snake_head.get_single().unwrap();
        apple
            .iter()
            .filter_map(|(apple_id, apple_transform)| {
                (head_transform.translation.x == apple_transform.translation.x
                    && head_transform.translation.y == apple_transform.translation.y)
                    .then_some(EatAppleEvent(apple_id))
            })
            .for_each(|ev| eat_apple_ev.send(ev));
    });
}

pub fn move_eaten_apple(
    mut eat_apple_ev: EventReader<EatAppleEvent>,
    mut apple: Query<&mut Transform, With<Apple>>,
    distribution: Res<ExcludeSnakeDistribution>,
    mut win_ev: EventWriter<WinEvent>,
) {
    let mut rng = rand::thread_rng();
    for EatAppleEvent(apple_id) in eat_apple_ev.read().copied() {
        if let Some(pos) = rng.sample(&*distribution) {
            apple.get_mut(apple_id).unwrap().translation = pos.into_snake_position();
        } else {
            win_ev.send(WinEvent);
        }
    }
}

pub fn elongate_body(
    mut eat_apple_ev: EventReader<EatAppleEvent>,
    snake: Query<Entity, With<Snake>>,
    head: Query<(Entity, &Transform, &MoveDirection), With<SnakeHead>>,
    mut piece_after_head: Query<&mut PrevId>,
    mut commands: Commands,
    sprites: Res<Sprites>,
    mut elongate_ev: EventWriter<ElongateEvent>,
) {
    for _ in eat_apple_ev.read() {
        commands
            .entity(snake.get_single().unwrap())
            .with_children(|snake| {
                let (
                    head_id,
                    Transform {
                        translation,
                        rotation,
                        ..
                    },
                    head_direction,
                ) = head.get_single().unwrap();
                piece_after_head
                    .iter_mut()
                    .find(|prev_id| prev_id.0 == head_id)
                    .unwrap()
                    .0 = snake
                    .spawn((
                        SnakeTail::Middle,
                        SnakePiece,
                        PrevId(head_id),
                        *head_direction,
                        make_sprite(
                            sprites.tail_middle(),
                            {
                                let mut t = *translation;
                                t.z = -1.0;
                                t
                            },
                            Some(*rotation),
                        ),
                    ))
                    .id();
            });
        elongate_ev.send(ElongateEvent);
    }
}
