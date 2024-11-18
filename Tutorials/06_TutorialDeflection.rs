// Tutorial: Deflection
use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0; // m/s
const PASSES: i32 = 10; // passes to project the target position (the more passes the more precise)

pub struct Ship {
    last_target_velocity : Vec2
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            last_target_velocity : vec2(0.0, 0.0)
        }
    }

    pub fn tick(&mut self) {
        let current_dir = target() - position();
        let target_acceleration = (target_velocity() - self.last_target_velocity) * 60.0;
        draw_line(target(), target() + target_acceleration, 0x00ff00);
        let mut projected = current_dir;
        for _i in 0..PASSES { // repeatedly projecting with the new distance
            let t = projected.length() / BULLET_SPEED;
            projected = target() + target_velocity() * t + (1.0/2.0) * target_acceleration * t * t;
        }
        let projected_angle = (projected - position()).angle();
        let turning_angle = angle_diff(heading(), projected_angle);
        let projected_angular_speed = angle_diff(projected_angle, (current_dir - position()).angle()) / (projected.length() / BULLET_SPEED);

        draw_triangle(projected, 10.0, 0xf1f100);
        draw_line(position(), vec2(heading().cos(), heading().sin())*6000.0, 0xff0000);
        debug!("projected angular speed: {:.2}", projected_angular_speed.abs());

        turn((turning_angle * turning_angle * 300.0 + projected_angular_speed.abs() + 0.01) * turning_angle.signum());

        if turning_angle.abs() < PI/360.0 {
            fire(0);
        }
        self.last_target_velocity = target_velocity();
    }
}
