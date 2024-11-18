use oort_api::prelude::*;

pub fn normal_vector(vector: Vec2) -> Vec2 {
    return vec2(-vector.y, vector.x);
}

pub fn get_dir_from_heading(heading: f64) -> Vec2 {
    return vec2(heading.cos(), heading.sin());
}

// pub trait NumberExtensions {
//     fn abs_min(&self, other: f64) -> Self;
// }

// impl NumberExtensions for f64 {
//     fn abs_min(&self, other: f64) -> f64 {
//         if self.abs() < other.abs() || (self.abs() == other.abs() && *self > other) {
//             *self
//         } else {
//             other
//         }
//     }
// }