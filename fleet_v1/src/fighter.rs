use core::f64;

// Fighter
use oort_api::prelude::*;
use oort_api::prelude::maths_rs::num::Base;

use crate::navigation;
use crate::settings;
use crate::ballistics;
use crate::utility;
use crate::radar;

// Crusing mode Stuff
const C_DIST: f64 = 10000.0;

pub struct Fighter {
    // Ships
    last_target_velocity : Vec2,
    // Radar
    scans_to_do : i32,
    best_target : f64,
}

impl Fighter {
    pub fn new() -> Fighter {
        let mut f = Fighter {
            last_target_velocity : Vec2::zero(),
            scans_to_do : 0,
            best_target : f64::MAX
        };
        radar::reset(&mut f.scans_to_do);
        f
    }

    pub fn tick(&mut self) {
        if self.scans_to_do <= 0 {
            if let Some(contact) = scan() {
                if contact.class == Class::Missile { return; }

                radar::track(contact.position, contact.velocity);

                // -- targeting mode --
                self.fighter_targeting(contact.position, contact.velocity);
            } else {
                // -- activate scanning mode --
                radar::reset(&mut self.scans_to_do);

                accelerate(utility::get_dir_from_heading(heading()) * max_forward_acceleration());
            }
        } else {
            // -- scanning mode --
            debug!("Scanning...");

            radar::slice_scan(&mut self.scans_to_do, &mut self.best_target);

            accelerate(utility::get_dir_from_heading(heading()) * max_forward_acceleration());

            // Deactivate boost if no target in sight
            deactivate_ability(Ability::Boost);
        }
    }
    pub fn fighter_targeting(&mut self, target: Vec2, target_velocity: Vec2) {
        let target_acceleration = self.get_target_acceleration(target_velocity);

        let relative_position = target - position();
        let relative_velocity = target_velocity - velocity();

        let relative_directional_velocity = relative_velocity.dot(relative_position.normalize());
        // main argument (use dogfight mode if the two spaceships are close or closing in)
        if reload_ticks(0) <= 5 && relative_directional_velocity < settings::BULLET_SPEED && relative_position.length() < C_DIST {
            // If bullets can realistically reach target, enter dogfight mode.
            self.dogfight_mode(target, target_velocity, target_acceleration);
            deactivate_ability(Ability::Boost);
        } else {
            let acceleration = navigation::calculate_prop_nav_acceleration(target, target_velocity);
            debug!("Crusing...");
            navigation::turn_to_static(angle_diff(heading(), acceleration.angle()));
            accelerate(acceleration);
            activate_ability(Ability::Boost)
        }
        // fire missiles if looking in the right direction or close enough the missile can lock on
        if reload_ticks(1) == 0 &&
            (angle_diff(heading(), relative_position.angle()).abs() < settings::MISSILE_FIRE_ANGLE || relative_position.length() < settings::MISSILE_FIRE_DIST) {
            fire(1);
        }
    }
    pub fn dogfight_mode(&mut self, target: Vec2, target_velocity: Vec2, target_acceleration: Vec2) {
        // calculate angular speed of the target relative to the ship
        let target_angular_speed = navigation::get_angular_speed(target, target_velocity);

        // -- acceleration --
        let acceleration = navigation::calculate_dogfight_acceleration(target, target_angular_speed);

        accelerate(acceleration);

        // -- turning and aiming --
        let lead_position = ballistics::lead_from_self(target, target_velocity, target_acceleration, settings::BULLET_SPEED, settings::ENEMY_SIZE);
        
        let turning_angle = angle_diff(heading(), (lead_position - position()).angle());
        
        navigation::turn(turning_angle, target_angular_speed);

        // -- gun logic --
        let angular_target_size = ballistics::get_angular_target_size(settings::ENEMY_SIZE, lead_position);

        if turning_angle.abs() <= angular_target_size / 2.0 {
            fire(0);
        }

        // -- debug stuff --
        debug!("total acceleration: {:.2}", acceleration.length());
        debug!("velocity: {:.2}", velocity().length());
        debug!("target dist: {:.2}", (target - position()).length());

        draw_line(position(), position() + utility::get_dir_from_heading(heading())*((lead_position - position()).length() + 50.0), 0xff0000); // aim vector
        draw_polygon(lead_position, settings::ENEMY_SIZE / 2.0, 10, 30.0, 0xf1f100); // lead indicator
    }

    pub fn get_target_acceleration(&mut self, target_velocity: Vec2) -> Vec2 { // CALL ONLY ONCE PER FRAME
        let acceleration = (target_velocity - self.last_target_velocity) * 60.0; // Calculating target acceleration between this and last frame
        self.last_target_velocity = target_velocity;
        return acceleration;
    }
}
