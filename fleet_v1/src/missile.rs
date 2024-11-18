use core::f64;

// Missile
use oort_api::prelude::*;

use crate::navigation;
use crate::utility;
use crate::radar;

// Missile stuff
const EXPLOSION_RANGE: f64 = 30.0;

pub struct Missile {
    // Radar
    scans_to_do : i32,
    best_target : f64,
    // Missiles
    original_angle : f64
}

impl Missile {
    pub fn new() -> Missile {
        let mut m = Missile {
            scans_to_do : 0,
            best_target : f64::MAX,
            original_angle : heading()
        };
        radar::reset(&mut m.scans_to_do);
        return m;
    }

    pub fn tick(&mut self) {
        if self.scans_to_do <= 0 {
            if let Some(contact) = scan() {
                if contact.class == Class::Missile { return; }

                radar::track(contact.position, contact.velocity);

                // -- targeting mode --
                self.missile_targeting(contact.position, contact.velocity);
            } else {
                // -- activate scanning mode --
                radar::reset(&mut self.scans_to_do);
                
                // always orient to original angle (mostly for missiles)
                accelerate(utility::get_dir_from_heading(self.original_angle) * max_forward_acceleration() / 2.0);
                navigation::turn_to_static(self.original_angle);
            }
        } else {
            // -- scanning mode --
            debug!("Scanning...");

            radar::slice_scan(&mut self.scans_to_do, &mut self.best_target);

            // always orient to original angle (mostly for missiles)
            accelerate(utility::get_dir_from_heading(self.original_angle) * max_forward_acceleration() / 2.0);
            navigation::turn_to_static(self.original_angle);
            
            // Deactivate boost if no target in sight
            deactivate_ability(Ability::Boost);
        }
    }
    pub fn missile_targeting(&mut self, target: Vec2, target_velocity: Vec2) {
        let acceleration = navigation::calculate_prop_nav_acceleration(target, target_velocity);
        debug!("acceleration: {}", acceleration.length());

        let turning_angle = angle_diff(heading(), acceleration.angle());
        navigation::turn_to_static(turning_angle);
        accelerate(acceleration);
        //debug!("turning angle = {}", turning_angle);
        if turning_angle < PI / 16.0 { // activate boost as soon as roughly looking at the target
            activate_ability(Ability::Boost)
        } else {
            deactivate_ability(Ability::Boost)
        }
        if (target - position()).length() < EXPLOSION_RANGE {
            explode();
        }
    }
}