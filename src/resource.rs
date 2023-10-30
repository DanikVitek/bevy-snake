use bevy::prelude::*;
use rand::prelude::*;

use crate::{FIELD_HEIGHT, FIELD_WIDTH};

#[derive(Default, Resource)]
pub struct ExcludeSnakeDistribution {
    pub positions: Vec<IVec2>,
}

impl Distribution<IVec2> for ExcludeSnakeDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> IVec2 {
        const RANGE_X: std::ops::Range<i32> = -FIELD_WIDTH / 2..FIELD_WIDTH / 2;
        const RANGE_Y: std::ops::Range<i32> = -FIELD_HEIGHT / 2..FIELD_HEIGHT / 2;
        let mut pos = IVec2::new(rng.gen_range(RANGE_X), rng.gen_range(RANGE_Y));
        while self.positions.contains(&pos) {
            pos = IVec2::new(rng.gen_range(RANGE_X), rng.gen_range(RANGE_Y));
        }
        pos
    }
}
