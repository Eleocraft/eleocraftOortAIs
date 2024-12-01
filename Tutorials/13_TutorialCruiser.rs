// Tutorial: Cruiser
use oort_api::prelude::*;
use oort_api::prelude::maths_rs::num::Base;

// Enemy Stuff
const ENEMY_SIZE: f64 = 25.0;

// Missile stuff
const N_FACTOR: f64 = 4.0;
const EXPLOSION_RANGE: f64 = 30.0;
const MISSILE_TURNING_RATE_MULT: f64 = 10.0;
const MISSILE_FIRE_DIST: f64 = 6000.0;
const MISSILE_FIRE_ANGLE: f64 = PI * 0.25;

// Prediction stuff
const TURRET_BULLET_SPEED: f64 = 2000.0; // m/s
const PASSES: i32 = 6;
const MAX_LEAD_TIME: f64 = 5.0;

// Turrets
const TURRET_OFFSET: f64 = 30.0;

// Radar stuff
const SCAN_SECTIONS: i32 = 10;
const SCAN_RANGE: f64 = 20000.0;
const TARGET_SCAN_RANGE_MULT: f64 = 0.02;
const TARGET_D_V_MULT: f64 = 0.04;

// Movement / Rotation stuff
const TURNING_SPEED_MULT: f64 = 400.0;
const MIN_TARGET_ANGLE_TO_ACCELERATE: f64 = PI * 0.5;
const MAX_FORWARD_DIST: f64 = 5000.0;

pub struct Ship {
    // Ships
    last_target_velocity : Vec2,
    // Radar
    scans_to_do : i32,
    closest_ship_angle : f64,
    // Missiles
    original_angle : f64
}

impl Ship {
    pub fn new() -> Ship {
        set_radar_width(2.0 * PI / SCAN_SECTIONS as f64);
        set_radar_min_distance(0.0);
        set_radar_max_distance(SCAN_RANGE);
        return Ship {
            last_target_velocity : Vec2::zero(),
            scans_to_do : SCAN_SECTIONS,
            closest_ship_angle : f64::MAX,
            original_angle : heading()
        };
    }

    pub fn tick(&mut self) {
        if class() == Class::Missile || class() == Class::Torpedo {
            self.missile_tick();
        } else if class() == Class::Cruiser {
            self.cruiser_tick();
        }
    }

    fn cruiser_tick(&mut self) {
        if self.scans_to_do <= 0 {
            if let Some(contact) = scan() {
                let target_distance = (contact.position - position()).length();
                let relative_speed = (contact.velocity - velocity()).length();
                let target_angle = (contact.position - position()).angle();

                // -- update radar to keep enemy in sight --
                let target_scan_range = ENEMY_SIZE + TARGET_SCAN_RANGE_MULT * target_distance + TARGET_D_V_MULT * relative_speed;
                set_radar_heading(target_angle);
                set_radar_width(target_scan_range / target_distance);
                set_radar_min_distance(target_distance - target_scan_range / 2.0);
                set_radar_max_distance(target_distance + target_scan_range / 2.0);


                // -- targeting mode --
                self.cruiser_targeting(contact.position, contact.velocity);
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
    fn missile_tick(&mut self) {
        if self.scans_to_do <= 0 {
            if let Some(contact) = scan() {
                let target_distance = (contact.position - position()).length();
                let relative_speed = (contact.velocity - velocity()).length();
                let target_angle = (contact.position - position()).angle();

                // -- update radar to keep enemy in sight --
                let target_scan_range = ENEMY_SIZE + TARGET_SCAN_RANGE_MULT * target_distance + TARGET_D_V_MULT * relative_speed;
                set_radar_heading(target_angle);
                set_radar_width(target_scan_range / target_distance);
                set_radar_min_distance(target_distance - target_scan_range / 2.0);
                set_radar_max_distance(target_distance + target_scan_range / 2.0);


                // -- targeting mode --
                self.missile_targeting(contact.position, contact.velocity);
            } else {
                // -- activate scanning mode --
                self.scans_to_do = SCAN_SECTIONS;
                set_radar_min_distance(0.0);
                set_radar_max_distance(SCAN_RANGE);
                set_radar_width(2.0 * PI / SCAN_SECTIONS as f64);
                
                // always orient to original angle (mostly for missiles)
                accelerate(Self::get_dir_from_heading(self.original_angle) * max_forward_acceleration() / 2.0);
                turn(self.original_angle);
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
            // HARDCODED 0 angle
            accelerate(Self::get_dir_from_heading(self.original_angle) * max_forward_acceleration() / 2.0);
            turn(self.original_angle);
            
            // Deactivate boost if no target in sight
            deactivate_ability(Ability::Boost);
        }
    }
    fn missile_targeting(&mut self, target: Vec2, target_velocity: Vec2) {
        let acceleration = Self::calculate_prop_nav_acceleration(target, target_velocity);
        debug!("acceleration: {}", acceleration.length());

        let turning_angle = angle_diff(heading(), acceleration.angle());
        turn(turning_angle * MISSILE_TURNING_RATE_MULT);
        accelerate(acceleration);
        
        if turning_angle < PI / 16.0 { // activate boost as soon as roughly looking at the target
            activate_ability(Ability::Boost)
        } else {
            deactivate_ability(Ability::Boost)
        }
        if (target - position()).length() < EXPLOSION_RANGE {
            explode();
        }
    }
    fn cruiser_targeting(&mut self, target: Vec2, target_velocity: Vec2) {
        let target_acceleration = self.get_target_acceleration(target_velocity);

        // calculate angular speed of the target relative to the ship
        let target_angular_speed = Self::get_angular_speed(target, target_velocity);

        // -- acceleration --
        let acceleration = Self::calculate_dogfight_acceleration(target);

        accelerate(acceleration);

        // -- turning and aiming --
        let turret_lead_position = Self::lead(target, target_velocity, target_acceleration, TURRET_BULLET_SPEED, position());

        let turning_angle = angle_diff(heading(), (target - position()).angle());

        turn(((turning_angle * turning_angle * TURNING_SPEED_MULT).abs() + target_angular_speed.abs()) * turning_angle.signum());

        // -- gun logic --
        fire(1);
        fire(2);
        fire(3);

        let turret_angle = (turret_lead_position - position()).angle();

        aim(0, turret_angle);
        fire(0);

        // -- debug stuff --
        debug!("total acceleration: {:.2}", acceleration.length());
        debug!("velocity: {:.2}", velocity().length());
        debug!("target dist: {:.2}", (target - position()).length());

        draw_line(position(), position() + Self::get_dir_from_heading(heading())*((turret_lead_position - position()).length() + 50.0), 0xff0000); // aim vector
        draw_polygon(turret_lead_position, ENEMY_SIZE / 2.0, 10, 30.0, 0xf1f100); // lead indicator
    }

    fn get_target_acceleration(&mut self, target_velocity: Vec2) -> Vec2 { // CALL ONLY ONCE PER FRAME
        let acceleration = (target_velocity - self.last_target_velocity) * 60.0; // Calculating target acceleration between this and last frame
        self.last_target_velocity = target_velocity;
        return acceleration;
    }

    // STATIC FUNCTIONS
    fn calculate_prop_nav_acceleration(target: Vec2, target_velocity: Vec2) -> Vec2 {
        let relative_position = target - position();
        let relative_velocity = target_velocity - velocity();
        // V_los = dot(dv, dp) / dp.magnitude
        let relative_directional_velocity = relative_velocity.dot(relative_position.normalize());
        // a_n = N * lambda * V_los
        let perpendicular_acceleration = N_FACTOR * Self::get_angular_speed(target, target_velocity) * relative_directional_velocity;
        // a_los = sqrt(a_max²-a_n²)
        let los_acceleration = if perpendicular_acceleration.abs() > max_forward_acceleration() { 0.0 /* overflow for high perpendicular acceleration */ } else {
            (max_forward_acceleration()*max_forward_acceleration()-perpendicular_acceleration*perpendicular_acceleration).sqrt()
        };
        // a = los * a_los + n * a_n
        return relative_position.normalize() * los_acceleration + Self::normal_vector(relative_position.normalize()) * perpendicular_acceleration;
    }

    fn calculate_dogfight_acceleration(target: Vec2) -> Vec2 { // Calculate the acceleration in close encounter mode
        if angle_diff(heading(), (target - position()).angle()).abs() > MIN_TARGET_ANGLE_TO_ACCELERATE {
            return Vec2::zero(); // return 0 if target is not in front
        }

        let heading_dir = Self::get_dir_from_heading(heading());
        let forward_acceleration = f64::min(max_forward_acceleration() * (target - position()).length() / MAX_FORWARD_DIST, max_forward_acceleration());
        return heading_dir * forward_acceleration;
    }

    fn get_angular_speed(target: Vec2, target_velocity: Vec2) -> f64 { // Get the speed by which the target rotates "around" the ship (in radians)
        let target_perpendicular_vector = Self::normal_vector((target - position()).normalize());
        let target_relative_speed = target_perpendicular_vector.dot(target_velocity);
        let self_relative_speed = target_perpendicular_vector.dot(velocity());
        return (self_relative_speed - target_relative_speed) / (target - position()).length();
    }

    fn get_angular_target_size(enemy_size: f64, target: Vec2) -> f64 { // get the size of a target on the periferal view of a ship (in radians)
        return enemy_size / (target - position()).length();
    }

    fn get_bullet_time(relative_position: Vec2, bullet_speed: f64) -> f64 { // Calculate the actual speed of bullets (taking into account ship velocity)
        return relative_position.length() / (bullet_speed + velocity().dot(relative_position.normalize()));
    }

    fn lead(target: Vec2, target_velocity : Vec2, target_acceleration: Vec2, bullet_speed: f64, origin_position: Vec2) -> Vec2 {
        let mut target_projected = target;
        for _i in 0..PASSES { // repeatedly projecting with the new distance
            let t = Self::get_bullet_time(target_projected - origin_position, bullet_speed); // calculating time to encounter with given speed
            target_projected = target + target_velocity * t + 0.5 * target_acceleration * t * t; // Calculating lead to target
        }
        let self_projected_relative = velocity() * Self::get_bullet_time(target_projected - origin_position, bullet_speed);
        return target_projected - self_projected_relative;
    }

    fn normal_vector(vector: Vec2) -> Vec2 {
        return vec2(-vector.y, vector.x);
    }

    fn get_dir_from_heading(heading: f64) -> Vec2 {
        return vec2(heading.cos(), heading.sin());
    }
}