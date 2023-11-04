use bevy::prelude::*;

#[derive(Component)]
pub struct Snake;

#[derive(Component)]
pub struct SnakeHead;

#[derive(Component)]
pub struct SnakePiece;

#[derive(Component)]
pub enum SnakeTail {
    Middle,
    End,
}

#[derive(Clone, Copy, Component)]
pub struct PrevId(pub Entity);

#[derive(Clone, Copy, Default, Component, PartialEq, Eq)]
pub enum MoveDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Component)]
pub struct PrevDirection(pub Option<MoveDirection>);

#[derive(Component)]
pub struct Apple;

impl MoveDirection {
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

impl From<MoveDirection> for Quat {
    fn from(value: MoveDirection) -> Self {
        Quat::from_rotation_z(match value {
            MoveDirection::Up => 0.0,
            MoveDirection::Down => std::f32::consts::PI,
            MoveDirection::Left => std::f32::consts::FRAC_PI_2,
            MoveDirection::Right => std::f32::consts::FRAC_PI_2 * 3.0,
        })
    }
}

impl From<&MoveDirection> for Quat {
    #[inline(always)]
    fn from(value: &MoveDirection) -> Self {
        Quat::from(*value)
    }
}
