use oort_api::prelude::*;
use oort_api::prelude::maths_rs::num::Base;


// Frigate
use crate::navigation;
use crate::settings;
use crate::ballistics;
use crate::utility;
use crate::radar;

const TURRET_OFFSET: f64 = 10.0;

pub struct Frigate {
    // Ships
    last_target_velocity : Vec2,
    // Radar
    scans_to_do : i32,
    best_target : f64,
}

impl Frigate {
    pub fn new() -> Frigate {
        let mut f = Frigate {
            last_target_velocity : Vec2::zero(),
            scans_to_do : 0,
            best_target : f64::MAX,
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
                self.frigate_targeting(contact.position, contact.velocity);
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
    fn frigate_targeting(&mut self, target: Vec2, target_velocity: Vec2) {
        let target_acceleration = self.get_target_acceleration(target_velocity);

        // calculate angular speed of the target relative to the ship
        let target_angular_speed = navigation::get_angular_speed(target, target_velocity);

        // -- acceleration --
        let acceleration = navigation::calculate_dogfight_acceleration(target, target_angular_speed);

        accelerate(acceleration);

        // -- turning and aiming --
        let lead_position = ballistics::lead_from_self(target, target_velocity, target_acceleration, settings::RAILGUN_BULLET_SPEED, settings::ENEMY_SIZE);

        let right_turret_position = vec2(0.0, TURRET_OFFSET).rotate(heading()) + position();
        let left_turret_position = vec2(0.0, -TURRET_OFFSET).rotate(heading()) + position();
        
        let right_turret_lead_position = ballistics::lead(target, target_velocity, target_acceleration, settings::BULLET_SPEED, settings::ENEMY_SIZE, right_turret_position);
        let left_turret_lead_position = ballistics::lead(target, target_velocity, target_acceleration, settings::BULLET_SPEED, settings::ENEMY_SIZE, left_turret_position);

        let turning_angle = angle_diff(heading(), (lead_position - position()).angle());

        navigation::turn(turning_angle, target_angular_speed);

        // -- gun logic --
        let angular_target_size = ballistics::get_angular_target_size(settings::ENEMY_SIZE, lead_position);

        if turning_angle.abs() <= angular_target_size / 2.0 {
            fire(0);
        }
        if turning_angle.abs() < settings::MISSILE_FIRE_ANGLE || (target - position()).length() < settings::MISSILE_FIRE_DIST {
            fire(3);
        }

        let right_turret_angle = (right_turret_lead_position - right_turret_position).angle();
        let left_turret_angle = (left_turret_lead_position - left_turret_position).angle();

        aim(1, right_turret_angle);
        aim(2, left_turret_angle);
        fire(1);
        fire(2);

        // -- debug stuff --
        debug!("total acceleration: {:.2}", acceleration.length());
        debug!("velocity: {:.2}", velocity().length());
        debug!("target dist: {:.2}", (target - position()).length());

        draw_line(position(), position() + utility::get_dir_from_heading(heading())*((lead_position - position()).length() + 50.0), 0xff0000); // aim vector
        draw_polygon(lead_position, settings::ENEMY_SIZE / 2.0, 10, 30.0, 0xf1f100); // lead indicator
        draw_polygon(right_turret_lead_position, settings::ENEMY_SIZE / 2.0, 10, 30.0, 0x00ff00); // right turret lead indicator
        draw_polygon(left_turret_lead_position, settings::ENEMY_SIZE / 2.0, 10, 30.0, 0x00ff00); // left turret lead indicator
    }

    fn get_target_acceleration(&mut self, target_velocity: Vec2) -> Vec2 { // CALL ONLY ONCE PER FRAME
        let acceleration = (target_velocity - self.last_target_velocity) * 60.0; // Calculating target acceleration between this and last frame
        self.last_target_velocity = target_velocity;
        return acceleration;
    }
}
