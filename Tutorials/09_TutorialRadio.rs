// Tutorial: Radio
use oort_api::prelude::*;
use oort_api::prelude::maths_rs::num::Base;

// Crusing mode Stuff
const MAX_VELOCITY_DIFF: f64 = 600.0;
const MAX_DIST: f64 = 6000.0;

// Enemy Stuff
const ENEMY_SIZE: f64 = 25.0;

// Prediction stuff
const BULLET_SPEED: f64 = 1000.0; // m/s
const PASSES: i32 = 6;
const MAX_LEAD_TIME: f64 = 5.0;

// Movement / Rotation stuff
const TURNING_SPEED_MULT: f64 = 200.0;
const MIN_TARGET_ANGLE_TO_ACCELERATE: f64 = PI * 0.5;
const MAX_FORWARD_DIST: f64 = 5000.0;
const MIN_LATERAL_DIST: f64 = 500.0;
const MAX_SPEED_DIFF: f64 = 800.0;

pub struct Ship {
    last_target_velocity : Vec2
}

impl Ship {
    pub fn new() -> Ship {
        return Ship {
            last_target_velocity : Vec2::zero()
        };
    }

    pub fn tick(&mut self) {
        set_radio_channel(2);
        if let Some(msg) = receive() {
            let contact_position = vec2(msg[0], msg[1]);
            let contact_velocity = vec2(msg[2], msg[3]);
            let to_contact = contact_position - position();

            // calculate target acceleration from last frame's data
            let contact_acceleration = self.get_target_acceleration(contact_velocity);
            
            let relative_velocity = velocity() - contact_velocity;
            let relative_directional_velocity = relative_velocity.dot((to_contact).normalize());
            if  relative_directional_velocity > -MAX_VELOCITY_DIFF && to_contact.length() < MAX_DIST {
                // If bullets can realistically reach target, enter dogfight mode.
                self.dogfight_mode(contact_position, contact_velocity, contact_acceleration);
            } else {
                // crusing mode (get back to target) this is a very simple barebones installation
                let acceleration = if relative_directional_velocity <= -MAX_VELOCITY_DIFF {
                    -relative_velocity.normalize() * 60.0
                } else {
                    to_contact.normalize() * 60.0
                };
                debug!("Crusing...");
                debug!("acceleration: {:.2}", acceleration.length());
                turn(angle_diff(heading(), acceleration.angle()));
                accelerate(acceleration);
            }
        }
        else {
            debug!("NO RADIO RECEIVED");
        }
    }

    pub fn dogfight_mode(&self, contact_position: Vec2, contact_velocity: Vec2, contact_acceleration: Vec2) {
        // calculate angular speed of the target relative to the ship
        let target_angular_speed = self.get_angular_speed(contact_position, contact_velocity);

        // -- acceleration --
        let acceleration = self.calculate_dogfight_acceleration(contact_position, contact_velocity, target_angular_speed);

        accelerate(acceleration);

        // -- turning and aiming --
        let lead_position = self.lead(contact_position, contact_velocity, contact_acceleration, |t| self.get_bullet_speed(t), PASSES);
        
        let turning_angle = angle_diff(heading(), (lead_position - position()).angle());

        turn(((turning_angle * turning_angle * TURNING_SPEED_MULT).abs() + target_angular_speed.abs()) * turning_angle.signum());

        // -- gun logic --
        let angular_target_size = self.get_angular_target_size(ENEMY_SIZE, lead_position);

        if turning_angle.abs() <= angular_target_size / 2.0 {
            fire(0);
        }

        // -- debug stuff --
        debug!("total acceleration: {:.2}", acceleration.length());
        debug!("velocity: {:.2}", velocity().length());
        debug!("target dist: {:.2}", (contact_position - position()).length());

        draw_line(position(), position() + Self::get_dir_from_heading(heading())*((lead_position - position()).length() + 50.0), 0xff0000); // aim vector
        draw_polygon(lead_position, ENEMY_SIZE / 2.0, 10, 30.0, 0xf1f100); // lead indicator
    }

    pub fn calculate_dogfight_acceleration(&self, target: Vec2, target_velocity: Vec2, target_angular_speed: f64) -> Vec2 { // Calculate the acceleration in close encounter mode
        if angle_diff(heading(), (target - position()).angle()).abs() > MIN_TARGET_ANGLE_TO_ACCELERATE {
            return Vec2::zero(); // return 0 if target is not in front
        }
        let relative_velocity = velocity() - target_velocity;
        if relative_velocity.length() > MAX_SPEED_DIFF {
            return -relative_velocity.normalize() * 60.0;
        }
        let heading_dir = Self::get_dir_from_heading(heading());
        let perpendicular_heading_dir = Self::normal_vector(heading_dir) * target_angular_speed.signum();
        let forward_acceleration = f64::min(max_forward_acceleration() * (target - position()).length() / MAX_FORWARD_DIST, max_forward_acceleration());
        let lateral_acceleration = f64::max(MIN_LATERAL_DIST - (target - position()).length(), 0.0) * max_lateral_acceleration() / MIN_LATERAL_DIST;
        return heading_dir * forward_acceleration + perpendicular_heading_dir * lateral_acceleration;
    }

    pub fn get_angular_speed(&self, target: Vec2, target_velocity: Vec2) -> f64 { // Get the speed by which the target rotates "around" the ship (in radians)
        let target_perpendicular_vector = (target - position()).normalize();
        let target_relative_speed = Self::normal_vector(target_perpendicular_vector).dot(target_velocity);
        let self_relative_speed = Self::normal_vector(target_perpendicular_vector).dot(velocity());
        return (self_relative_speed - target_relative_speed) / (target - position()).length();
    }

    pub fn get_angular_target_size(&self, enemy_size: f64, target: Vec2) -> f64 { // get the size of a target on the periferal view of a ship (in radians)
        return enemy_size / (target - position()).length();
    }

    pub fn get_bullet_speed(&self, target: Vec2) -> f64 { // Calculate the actual speed of bullets (taking into account ship velocity)
        return BULLET_SPEED + velocity().dot((target - position()).normalize());
    }

    pub fn get_target_acceleration(&mut self, target_velocity: Vec2) -> Vec2 { // CALL ONLY ONCE PER FRAME
        let acceleration = (target_velocity - self.last_target_velocity) * 60.0; // Calculating target acceleration between this and last frame
        self.last_target_velocity = target_velocity;
        return acceleration;
    }

    pub fn time_to_encounter(&self, target: Vec2, speed: f64) -> f64 { // Calculate the time to reach "target" if moving with "speed"
        return (target - position()).length() / speed;
    }

    pub fn lead<F>(&self, target: Vec2, target_velocity : Vec2, target_acceleration: Vec2, get_speed: F,  passes: i32) -> Vec2 where F: Fn(Vec2) -> f64 {
        let mut target_projected = target;
        for _i in 0..passes { // repeatedly projecting with the new distance
            let t = self.time_to_encounter(target_projected, get_speed(target_projected)); // calculating time to encounter with given speed
            target_projected = target + target_velocity * t + 0.5 * target_acceleration * t * t; // Calculating lead to target
        }
        let self_projected_relative = velocity() * self.time_to_encounter(target_projected, get_speed(target_projected));
        return target_projected - self_projected_relative;
    }

    // STATIC FUNCTIONS

    pub fn normal_vector(vector: Vec2) -> Vec2 {
        return vec2(-vector.y, vector.x);
    }

    pub fn get_dir_from_heading(heading: f64) -> Vec2 {
        return vec2(heading.cos(), heading.sin());
    }
}
