// Tutorial: Acceleration 2
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        // Hint: "target() - position()" returns a vector pointing towards the target.
        accelerate(target()-position());
    }
}
