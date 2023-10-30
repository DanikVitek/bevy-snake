use bevy::prelude::*;

#[derive(Clone, Copy, Event)]
pub struct UpdateDirectionEvent(pub Entity);

#[derive(Event)]
pub struct StepEvent;

#[derive(Clone, Copy, Event)]
pub struct EatAppleEvent(pub Entity);
