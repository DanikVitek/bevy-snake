use std::ops::RangeInclusive;

use bevy::{asset::LoadState, prelude::*};
use rand::prelude::*;

use crate::{FIELD_HEIGHT, FIELD_WIDTH};

#[derive(Default, Resource)]
pub struct ExcludeSnakeDistribution {
    pub positions: Vec<IVec2>,
}

impl Distribution<Option<IVec2>> for ExcludeSnakeDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<IVec2> {
        const RANGE_X: RangeInclusive<i32> = -FIELD_WIDTH / 2..=FIELD_WIDTH / 2;
        const RANGE_Y: RangeInclusive<i32> = -FIELD_HEIGHT / 2..=FIELD_HEIGHT / 2;
        RANGE_X
            .flat_map(|x| {
                RANGE_Y.filter_map(move |y| {
                    let pos = IVec2::new(x, y);
                    if self.positions.iter().any(|&exclude_pos| exclude_pos == pos) {
                        None
                    } else {
                        Some(pos)
                    }
                })
            })
            .choose(rng)
    }
}

#[derive(Default, Resource)]
pub struct Sprites {
    head: Option<Handle<Image>>,
    tail_middle: Option<Handle<Image>>,
    tail_turn: Option<Handle<Image>>,
    tail_end: Option<Handle<Image>>,
    apple: Option<Handle<Image>>,
}

impl Sprites {
    pub fn load(&mut self, asset_server: &AssetServer) {
        self.head = Some(asset_server.load("sprites/snake-head.png"));
        self.tail_middle = Some(asset_server.load("sprites/snake-body.png"));
        self.tail_turn = Some(asset_server.load("sprites/snake-body-turn.png"));
        self.tail_end = Some(asset_server.load("sprites/snake-end.png"));
        self.apple = Some(asset_server.load("sprites/apple.png"));
    }

    pub fn head(&self) -> Handle<Image> {
        self.head.clone().unwrap()
    }

    pub fn tail_middle(&self) -> Handle<Image> {
        self.tail_middle.clone().unwrap()
    }

    pub fn tail_turn(&self) -> Handle<Image> {
        self.tail_turn.clone().unwrap()
    }

    pub fn tail_end(&self) -> Handle<Image> {
        self.tail_end.clone().unwrap()
    }

    pub fn apple(&self) -> Handle<Image> {
        self.apple.clone().unwrap()
    }

    pub fn check_loaded(&self, asset_server: &AssetServer) -> bool {
        if let (Some(head), Some(tail_middle), Some(tail_turn), Some(tail_end), Some(apple)) = (
            &self.head,
            &self.tail_middle,
            &self.tail_turn,
            &self.tail_end,
            &self.apple,
        ) {
            asset_server.get_group_load_state(
                [head, tail_middle, tail_turn, tail_end, apple].map(Handle::id),
            ) == LoadState::Loaded
        } else {
            false
        }
    }
}
