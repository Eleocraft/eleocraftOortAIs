use oort_api::prelude::*;


use crate::settings;

pub fn get_angular_target_size(enemy_size: f64, target: Vec2) -> f64 { // get the size of a target on the periferal view of a ship (in radians)
    return enemy_size / (target - position()).length();
}

pub fn get_bullet_time(relative_position: Vec2, bullet_speed: f64) -> f64 { // Calculate the actual speed of bullets (taking into account ship velocity)
    return relative_position.length() / (bullet_speed + velocity().dot(relative_position.normalize()));
}

pub fn lead(target: Vec2, target_velocity : Vec2, target_acceleration: Vec2, bullet_speed: f64, origin_position: Vec2) -> Vec2 {
    let mut target_projected = target;
    for _i in 0..settings::PASSES { // repeatedly projecting with the new distance
        let t = get_bullet_time(target_projected - origin_position, bullet_speed); // calculating time to encounter with given speed
        target_projected = target + target_velocity * t + 0.5 * target_acceleration * t * t; // Calculating lead to target
    }
    let self_projected_relative = velocity() * get_bullet_time(target_projected - origin_position, bullet_speed);
    return target_projected - self_projected_relative;
}

pub fn lead_from_me(target: Vec2, target_velocity : Vec2, target_acceleration: Vec2, bullet_speed: f64) -> Vec2 {
    return lead(target, target_velocity, target_acceleration, bullet_speed, position());
}