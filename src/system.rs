use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::Rng;

use crate::component::{Direction, *};
use crate::resource::{ExcludeSnakeDistribution, Sprites};
use crate::{event::*, make_sprite, IntoSnakePosition};
use crate::{CELL_SIZE, VISIBLE_FIELD_HEIGHT, VISIBLE_FIELD_WIDTH};

pub fn move_snake(
    time: Res<Time>,
    mut snake: Query<&mut Snake>,
    mut head: Query<(Entity, &Direction, &mut PrevDirection, &mut Transform), With<SnakeHead>>,
    mut pieces: Query<(&Direction, &mut Transform), (Without<PrevDirection>, With<SnakeTail>)>,
    mut update_direction_ev: EventWriter<UpdateDirectionEvent>,
    mut step_ev: EventWriter<StepEvent>,
    mut elongate_ev: ResMut<Events<ElongateEvent>>,
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

    if elongate_ev.is_empty() {
        for (direction, mut transform) in pieces.iter_mut() {
            translate(&mut transform, *direction);
        }
    } else {
        elongate_ev.clear();
    }

    step_ev.send(StepEvent);
}

fn translate(transform: &mut Transform, direction: Direction) {
    match direction {
        Direction::Left => {
            transform.translation.x -= CELL_SIZE;
            if transform.translation.x < -VISIBLE_FIELD_WIDTH / 2.0 {
                transform.translation.x = VISIBLE_FIELD_WIDTH / 2.0 - CELL_SIZE;
            }
        }
        Direction::Right => {
            transform.translation.x += CELL_SIZE;
            if transform.translation.x > VISIBLE_FIELD_WIDTH / 2.0 {
                transform.translation.x = -VISIBLE_FIELD_WIDTH / 2.0 + CELL_SIZE;
            }
        }
        Direction::Up => {
            transform.translation.y += CELL_SIZE;
            if transform.translation.y > VISIBLE_FIELD_HEIGHT / 2.0 {
                transform.translation.y = -VISIBLE_FIELD_HEIGHT / 2.0 + CELL_SIZE;
            }
        }
        Direction::Down => {
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
    for _ in step_ev.iter() {
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
    snake: Query<&Snake>,
    tail: Query<&PrevId>,
    mut directions: ParamSet<(
        Query<&Direction>,
        Query<(Entity, &PrevId, &mut Direction), With<SnakeTail>>,
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

pub fn control_snake(
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

pub fn rotate_snake_sprite(
    mut update_direction_ev: EventReader<UpdateDirectionEvent>,
    mut sprite: Query<(&mut Transform, &Direction)>,
) {
    for UpdateDirectionEvent(id) in update_direction_ev.iter().copied() {
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
    for _ in step_ev.iter() {
        let head_transform = snake_head.get_single().unwrap();
        for (apple_id, apple_transform) in apple.iter() {
            if head_transform.translation == apple_transform.translation {
                eat_apple_ev.send(EatAppleEvent(apple_id));
            }
        }
    }
}

pub fn move_eaten_apple(
    mut eat_apple_ev: EventReader<EatAppleEvent>,
    mut apple: Query<&mut Transform, With<Apple>>,
    distribution: Res<ExcludeSnakeDistribution>,
    mut win_ev: EventWriter<WinEvent>,
) {
    let mut rng = rand::thread_rng();
    for EatAppleEvent(apple_id) in eat_apple_ev.iter().copied() {
        if let Some(pos) = rng.sample(&*distribution) {
            let mut apple_transform = apple.get_mut(apple_id).unwrap();
            let Vec2 { x, y } = pos.into_snake_position();
            apple_transform.translation = Vec3::new(x, y, 0.0);
        } else {
            win_ev.send(WinEvent);
        }
    }
}

pub fn elongate_body(
    mut eat_apple_ev: EventReader<EatAppleEvent>,
    snake: Query<Entity, With<Snake>>,
    head: Query<(Entity, &Transform, &Direction), With<SnakeHead>>,
    mut piece_after_head: Query<&mut PrevId>,
    mut commands: Commands,
    sprites: Res<Sprites>,
    mut elongate_ev: EventWriter<ElongateEvent>,
) {
    for _ in eat_apple_ev.iter() {
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
                        make_sprite(sprites.tail_middle(), *translation, Some(*rotation)),
                    ))
                    .id();
            });
        elongate_ev.send(ElongateEvent);
    }
}
