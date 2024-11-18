// Tutorial: Guns
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    // Uncomment me, then press Ctrl-Enter (Cmd-Enter on Mac) to upload the code.
    pub fn tick(&mut self) {
        fire(0);
        accelerate(vec2(1.0, 0.0) * 500.0);
    }
}
