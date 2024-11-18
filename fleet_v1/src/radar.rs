use oort_api::prelude::*;

use crate::settings;

pub fn track(target: Vec2, target_velocity: Vec2) {
    let target_distance = (target - position()).length();
    let relative_speed = (target_velocity - velocity()).length();
    let target_angle = (target - position()).angle();
    
    // -- update radar to keep enemy in sight --
    let target_scan_range = settings::ENEMY_SIZE + settings::TARGET_SCAN_RANGE_MULT * target_distance + settings::TARGET_D_V_MULT * relative_speed;
    set_radar_heading(target_angle);
    set_radar_width(target_scan_range / target_distance);
    set_radar_min_distance(target_distance - target_scan_range / 2.0);
    set_radar_max_distance(target_distance + target_scan_range / 2.0);
}

pub fn slice_scan(scans_to_do: &mut i32, best_target: &mut f64) {
    
    if let Some(contact) = scan() {
        if angle_diff(heading(), (contact.position - position()).angle()).abs() < best_target.abs() {
            *best_target = angle_diff(heading(), (contact.position - position()).angle());
        }
    }
    *scans_to_do -= 1;
    if *scans_to_do > 0 {
        set_radar_heading(radar_heading() + radar_width());
    } else {
        set_radar_heading(heading() + *best_target);
        *best_target = f64::MAX;
    }
}

pub fn reset(scans_to_do: &mut i32) {
    *scans_to_do = settings::SCAN_SECTIONS;
    set_radar_min_distance(0.0);
    set_radar_max_distance(f64::MAX);
    set_radar_width(TAU / settings::SCAN_SECTIONS as f64);
}