// Tutorial: Missiles
// https://en.wikipedia.org/wiki/Proportional_navigation
use oort_api::prelude::*;
use oort_api::prelude::maths_rs::num::Base;

// Radar stuff
const SCAN_SECTIONS: i32 = 4;
const SCAN_RANGE: f64 = 20000.0;
const TARGET_SCAN_RANGE: f64 = 50.0;

// Missile stuff
const N_FACTOR: f64 = 4.0;
const EXPLOSION_RANGE: f64 = 30.0;
const MISSILE_TURNING_RATE_MULT: f64 = 10.0;

pub struct Ship {
    last_target_velocity : Vec2,
    scans_to_do : i32,
    closest_ship_angle : f64
}

impl Ship {
    pub fn new() -> Ship {
        set_radar_width(2.0 * PI / SCAN_SECTIONS as f64);
        set_radar_min_distance(0.0);
        set_radar_max_distance(SCAN_RANGE);
        return Ship {
            last_target_velocity : Vec2::zero(),
            scans_to_do : SCAN_SECTIONS,
            closest_ship_angle : f64::MAX
        };
    }
    
    pub fn tick(&mut self) {
        if class() == Class::Fighter {
            fire(1); // If this is a fighter simply fire a missile asap
        }
        if self.scans_to_do <= 0 {
            // -- targeting mode --
            if let Some(contact) = scan() {
                let relative_position = contact.position - position();
                let relative_velocity = contact.velocity - velocity();

                set_radar_heading(relative_position.angle());
                set_radar_width(TARGET_SCAN_RANGE / relative_position.length());
                set_radar_min_distance(relative_position.length() - TARGET_SCAN_RANGE / 2.0);
                set_radar_max_distance(relative_position.length() + TARGET_SCAN_RANGE / 2.0);

                if class() == Class::Missile {
                    self.missile_targeting(contact.position, contact.velocity);
                } else if class() == Class::Fighter {
                    self.fighter_targeting(contact.position, contact.velocity);
                }
            } else {
                // -- activate scanning mode --
                self.scans_to_do = SCAN_SECTIONS;
                set_radar_min_distance(0.0);
                set_radar_max_distance(SCAN_RANGE);
                set_radar_width(2.0 * PI / SCAN_SECTIONS as f64);
            }
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
    pub fn missile_targeting(&mut self, target: Vec2, target_velocity: Vec2) {

        let relative_position = target - position();
        let relative_velocity = target_velocity - velocity();

        // V_los = dot(dv, dp) / dp.magnitude
        let relative_directional_velocity = relative_velocity.dot(relative_position.normalize());
        // a_n = N * lambda * V_los
        let perpendicular_acceleration = N_FACTOR * Self::get_angular_speed(target, target_velocity) * relative_directional_velocity;
        // a_los = sqrt(a_max²-a_n²)
        let los_acceleration = (max_forward_acceleration()*max_forward_acceleration()-perpendicular_acceleration*perpendicular_acceleration).sqrt();
        // a = los * a_los + n * a_n
        let acceleration = relative_position.normalize() * los_acceleration + Self::normal_vector(relative_position.normalize()) * perpendicular_acceleration;
        debug!("acceleration: {}", acceleration.length());


        let turning_angle = angle_diff(heading(), acceleration.angle());
        turn(turning_angle * MISSILE_TURNING_RATE_MULT);
        accelerate(acceleration);
        if (turning_angle < PI / 16.0) {
            activate_ability(Ability::Boost)
        }
        if relative_position.length() < EXPLOSION_RANGE {
            explode();
        }
    }
    pub fn fighter_targeting(&mut self, target: Vec2, target_velocity: Vec2) {
        // Very simplified dogfight mode (to be replaced with actual dogfight mode after missiles work well enough)
                
        let relative_position = target - position();
        // -- update radar to keep enemy in sight --
        set_radar_heading(relative_position.angle());
        set_radar_width(TARGET_SCAN_RANGE / relative_position.length());
        set_radar_min_distance(relative_position.length() - TARGET_SCAN_RANGE / 2.0);
        set_radar_max_distance(relative_position.length() + TARGET_SCAN_RANGE / 2.0);

        let turning_angle = angle_diff(heading(), relative_position.angle());

        turn(turning_angle * 60.0);

        accelerate(relative_position.normalize() * max_forward_acceleration());
    }
    // STATIC FUNCTIONS
    pub fn get_angular_speed(target: Vec2, target_velocity: Vec2) -> f64 { // Get the speed by which the target rotates "around" the ship (in radians)
        let target_perpendicular_vector = Self::normal_vector((target - position()).normalize());
        let target_relative_speed = target_perpendicular_vector.dot(target_velocity);
        let self_relative_speed = target_perpendicular_vector.dot(velocity());
        return (self_relative_speed - target_relative_speed) / (target - position()).length();
    }

    pub fn time_to_encounter(target: Vec2, speed: f64) -> f64 { // Calculate the time to reach "target" if moving with "speed"
        return (target - position()).length() / speed;
    }

    pub fn lead<F>(target: Vec2, target_velocity : Vec2, target_acceleration: Vec2, get_speed: F,  passes: i32) -> Vec2 where F: Fn(Vec2) -> f64 {
        let mut target_projected = target;
        for _i in 0..passes { // repeatedly projecting with the new distance
            let t = Self::time_to_encounter(target_projected, get_speed(target_projected)); // calculating time to encounter with given speed
            target_projected = target + target_velocity * t + 0.5 * target_acceleration * t * t; // Calculating lead to target
        }
        let self_projected_relative = velocity() * Self::time_to_encounter(target_projected, get_speed(target_projected));
        return target_projected - self_projected_relative;
    }

    pub fn normal_vector(vector: Vec2) -> Vec2 {
        return vec2(-vector.y, vector.x);
    }

    pub fn get_dir_from_heading(heading: f64) -> Vec2 {
        return vec2(heading.cos(), heading.sin());
    }
}
