use bevy::prelude::*;

#[derive(Component)]
pub struct Snake {
    pub next_move: Timer,
}

#[derive(Component)]
pub struct SnakeHead;

#[derive(Component)]
pub struct SnakePiece;

#[derive(Component)]
pub struct SnakeTail;

#[derive(Clone, Copy, Component)]
pub struct PrevId(pub Entity);

#[derive(Clone, Copy, Default, Component, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Component)]
pub struct PrevDirection(pub Option<Direction>);

#[derive(Component)]
pub struct Apple;

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
