use oort_api::prelude::*;

// Missile stuff
pub const N_FACTOR: f64 = 4.0;
pub const MISSILE_FIRE_DIST: f64 = 6000.0; // missiles are fired if either in range or pointing towards target
pub const MISSILE_FIRE_ANGLE: f64 = PI * 0.25;

// Prediction stuff
pub const BULLET_SPEED: f64 = 1000.0; // m/s
pub const CRUISER_BULLET_SPEED: f64 = 2000.0; // m/s
pub const RAILGUN_BULLET_SPEED: f64 = 4000.0; // m/s
pub const PASSES: i32 = 8;

// Dogfight movement stuff
pub const MAX_FORWARD_DIST: f64 = 5000.0;
pub const MIN_LATERAL_DIST: f64 = 500.0;

// Enemy Stuff
pub const ENEMY_SIZE: f64 = 25.0;

// Radar stuff
pub const TARGET_SCAN_RANGE_MULT: f64 = 0.02;
pub const TARGET_D_V_MULT: f64 = 0.04;
pub const SCAN_SECTIONS: i32 = 10;

// Basic navigation stuff
pub const ANGULAR_SPEED_PREDICTION_FACTOR: f64 = 0.2;