// Tutorial: Rotation
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        // Hint: "angle_diff(heading(), (target() - position()).angle())"
        // returns the direction your ship needs to turn to face the target.

        let targetAngle = (target()-position()).angle();
        let turningAngle = angle_diff(heading(), targetAngle);
        turn(turningAngle * turningAngle * turningAngle * 500.0);
        if (turningAngle.abs() < PI/8.0) {
            fire(0);
        }
    }
}
