use oort_api::prelude::*;

use crate::settings;
use crate::utility;

pub fn turn(angle: f64, target_angular_speed: f64) {
    // If the angle is so small that one step would be too much reduce acceleration
    let angular_acceleration = f64::min(2.0*angle.abs()/(TICK_LENGTH*TICK_LENGTH), max_angular_acceleration());

    let time_to_stop = (angular_velocity() + target_angular_speed).abs() / angular_acceleration;
    let dist_to_stop = (angular_velocity().abs() * time_to_stop - 0.5 * angular_acceleration * time_to_stop * time_to_stop).abs();

    let biased_projected_angle = angle + settings::ANGULAR_SPEED_PREDICTION_FACTOR * target_angular_speed * time_to_stop;

    if dist_to_stop < biased_projected_angle.abs() {
        // accelerate towards angle
        torque(angular_acceleration * biased_projected_angle.signum());
    } else {
        // brake
        torque(-angular_acceleration * angular_velocity().signum());
    }
}

pub fn turn_to_static(angle: f64) {
    turn(angle, 0.0);
}

pub fn calculate_prop_nav_acceleration(target: Vec2, target_velocity: Vec2) -> Vec2 {
    let relative_position = target - position();
    let relative_velocity = target_velocity - velocity();
    // V_los = dot(dv, dp) / dp.magnitude
    let relative_directional_velocity = relative_velocity.dot(relative_position.normalize());
    // a_n = N * lambda * V_los
    let perpendicular_acceleration = settings::N_FACTOR * get_angular_speed(target, target_velocity) * relative_directional_velocity;
    // a_los = sqrt(a_max²-a_n²)
    let los_acceleration = if perpendicular_acceleration.abs() > max_forward_acceleration() {
        // overflow for high perpendicular acceleration
        return relative_velocity.normalize() * max_forward_acceleration();
    } else {
        (max_forward_acceleration()*max_forward_acceleration()-perpendicular_acceleration*perpendicular_acceleration).sqrt()
    };
    // a = los * a_los + n * a_n
    return relative_position.normalize() * los_acceleration - utility::normal_vector(relative_position.normalize()) * perpendicular_acceleration;
}

pub fn calculate_dogfight_acceleration(target: Vec2, target_angular_speed: f64) -> Vec2 { // Calculate the acceleration in close encounter mode

    let heading_dir = (target - position()).normalize();
    let perpendicular_heading_dir = utility::normal_vector(heading_dir) * -target_angular_speed.signum();
    let relative_forward_acceleration = f64::min((target - position()).length() / settings::MAX_FORWARD_DIST, 1.0);
    let relative_lateral_acceleration = f64::max(settings::MIN_LATERAL_DIST - (target - position()).length(), 0.0) / settings::MIN_LATERAL_DIST;
    
    let target_dir_angle = (heading_dir * relative_forward_acceleration + perpendicular_heading_dir * relative_lateral_acceleration).angle();

    return get_max_acceleration(target_dir_angle);
}

pub fn get_max_acceleration(direction_angle: f64) -> Vec2 {
    let relative_angle = direction_angle - heading();
    let relative_direction = utility::get_dir_from_heading(relative_angle);

    let coaxial_acceleration = if relative_direction.x >= 0.0 {
        max_forward_acceleration()
    } else {
        max_backward_acceleration()
    }.min(max_lateral_acceleration() * (0.5*PI - relative_angle.abs()).tan());

    let lateral_acceleration = relative_angle.abs().tan() * coaxial_acceleration;

    return vec2(coaxial_acceleration * relative_direction.x.signum(), lateral_acceleration * relative_direction.y.signum()).rotate(heading());
}

pub fn get_angular_speed(target: Vec2, target_velocity: Vec2) -> f64 { // Get the speed by which the target rotates "around" the ship (in radians)
    let target_perpendicular_vector = utility::normal_vector((target - position()).normalize());
    let target_relative_speed = target_perpendicular_vector.dot(target_velocity);
    let self_relative_speed = target_perpendicular_vector.dot(velocity());
    return (target_relative_speed - self_relative_speed) / (target - position()).length();
}