// Tutorial: Radar
use oort_api::prelude::*;
use oort_api::prelude::maths_rs::num::Base;


const ENEMY_SIZE: f64 = 25.0;

// Prediction stuff
const BULLET_SPEED: f64 = 1000.0; // m/s
const PASSES: i32 = 6;

// Radar stuff
const SCAN_SECTIONS: i32 = 10;
const SCAN_RANGE: f64 = 20000.0;
const TARGET_SCAN_RANGE: f64 = 50.0;

// Movement / Rotation stuff
const TURNING_SPEED_MULT: f64 = 200.0;
const MIN_TARGET_ANGLE_TO_ACCELERATE: f64 = PI * 0.8;
const MAX_FORWARD_DIST: f64 = 2500.0;
const MIN_LATERAL_DIST: f64 = 800.0;

pub struct Ship {
    last_target_velocity : Vec2,
    scans_to_do : i32,
    closest_ship_angle : f64
}

impl Ship {
    pub fn new() -> Ship {
        set_radar_width(2.0 * PI / SCAN_SECTIONS as f64);
        return Ship {
            last_target_velocity : Vec2::zero(),
            scans_to_do : SCAN_SECTIONS,
            closest_ship_angle : f64::MAX
        };
    }

    pub fn tick(&mut self) {
        if self.scans_to_do <= 0 {
            // -- targeting mode --
            self.targeting();
        } else {
            // -- scanning mode --
            debug!("Scanning...");
            if let Some(contact) = scan() {
                if angle_diff(heading(), (contact.position - position()).angle()).abs() < self.closest_ship_angle.abs() {
                    self.closest_ship_angle = angle_diff(heading(), (contact.position - position()).angle());
                }
            }
            self.scans_to_do -= 1;
            if self.scans_to_do > 0 {
                set_radar_heading(radar_heading() + radar_width());
            } else {
                set_radar_heading(heading() + self.closest_ship_angle);
                self.closest_ship_angle = f64::MAX;
            }
        }
    }

    pub fn targeting(&mut self) {
        if let Some(contact) = scan() {
            
            // -- update radar to keep enemy in sight --
            set_radar_heading((contact.position - position()).angle());
            set_radar_width(TARGET_SCAN_RANGE / (contact.position - position()).length());
            set_radar_min_distance((contact.position - position()).length() - TARGET_SCAN_RANGE / 2.0);
            set_radar_max_distance((contact.position - position()).length() + TARGET_SCAN_RANGE / 2.0);
            
            // calculate angular speed of the target relative to the ship
            let target_angular_speed = self.get_angular_speed(contact.position, contact.velocity);

            // -- acceleration --
            let acceleration = self.calculate_acceleration(contact.position, target_angular_speed);

            accelerate(acceleration);

            // -- turning and aiming --
            let projected = self.lead(contact.position, contact.velocity);

            let turning_angle = angle_diff(heading(), (projected - position()).angle());

            turn(((turning_angle * turning_angle * TURNING_SPEED_MULT).abs() + target_angular_speed.abs()) * turning_angle.signum());

            let angular_target_size = self.get_angular_target_size(ENEMY_SIZE, projected);

            // -- debug stuff --
            debug!("total acceleration: {:.2}", acceleration.length());
            debug!("velocity: {:.2}", velocity().length());
            debug!("target dist: {:0.2}", (contact.position - position()).length());

            draw_line(position(), position() + Self::get_dir_from_heading(heading())*3000.0, 0xff0000); // aim vector
            draw_polygon(projected, ENEMY_SIZE / 2.0, 10, 30.0, 0xf1f100); // lead indicator

            // -- gun logic --
            if turning_angle.abs() <= angular_target_size / 2.0 {
                fire(0);
            }
        }
        else {
            // -- activate scanning mode --
            self.scans_to_do = SCAN_SECTIONS;
            set_radar_min_distance(0.0);
            set_radar_max_distance(SCAN_RANGE);
            set_radar_width(2.0 * PI / SCAN_SECTIONS as f64);
        }
    }

    pub fn calculate_acceleration(&self, target: Vec2, target_angular_speed: f64) -> Vec2 {
        if angle_diff(heading(), (target - position()).angle()).abs() > MIN_TARGET_ANGLE_TO_ACCELERATE {
            debug!("No acceleration");
            return Vec2::zero(); // return 0 if target is not in view
        }

        let heading_dir = Self::get_dir_from_heading(heading());
        let perpendicular_heading_dir = Self::normal_vector(heading_dir) * target_angular_speed.signum();
        let forward_acceleration = f64::min(max_forward_acceleration() * (target - position()).length() / MAX_FORWARD_DIST, max_forward_acceleration());
        let lateral_acceleration = f64::max(MIN_LATERAL_DIST - (target - position()).length(), 0.0) * max_lateral_acceleration() / MIN_LATERAL_DIST;
        return heading_dir * forward_acceleration + perpendicular_heading_dir * lateral_acceleration;
    }

    pub fn get_angular_speed(&self, target: Vec2, target_velocity: Vec2) -> f64 {
        let target_perpendicular_vector = (target - position()).normalize();
        let target_relative_speed = Self::normal_vector(target_perpendicular_vector).dot(target_velocity);
        let self_relative_speed = Self::normal_vector(target_perpendicular_vector).dot(velocity());
        return (self_relative_speed - target_relative_speed) / (target - position()).length();
    }

    pub fn get_angular_target_size(&self, enemy_size: f64, target: Vec2) -> f64 {
        return enemy_size / (target - position()).length();
    }

    pub fn get_bullet_time(&self, target: Vec2) -> f64 {
        return (target - position()).length() / (BULLET_SPEED + velocity().dot(Self::get_dir_from_heading(heading())));
    }

    pub fn lead(&mut self, target: Vec2, target_velocity : Vec2) -> Vec2 {
        let current_dir = target - position();
        let target_acceleration = (target_velocity - self.last_target_velocity) * 60.0;
        let mut projected = current_dir;
        for _i in 0..PASSES { // repeatedly projecting with the new distance
            let t = self.get_bullet_time(projected);
            projected = target + target_velocity * t + 0.5 * target_acceleration * t * t;
        }
        self.last_target_velocity = target_velocity;

        let t = self.get_bullet_time(projected);
        let self_projected_relative = velocity() * t;
        return projected - self_projected_relative;
    }

    // STATIC FUNCTIONS

    pub fn normal_vector(vector: Vec2) -> Vec2 {
        return vec2(-vector.y, vector.x);
    }

    pub fn get_dir_from_heading(heading: f64) -> Vec2 {
        return vec2(heading.cos(), heading.sin());
    }
}
