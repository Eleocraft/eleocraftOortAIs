// Tutorial: Acceleration
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        // Hint: uncomment me
        accelerate(vec2(100.0, 0.0));
    }
}
