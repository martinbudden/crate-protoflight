#![allow(unused)]

use vqm::{Quaternionf32, RollPitchf32, Vector2df32};

/// A simple physics simulation of a multirotor's vertical motion.
pub struct MockMultirotorZ {
    pub altitude: f32,
    pub vertical_speed: f32,
    pub hover_throttle: f32,
    pub mass: f32,
    pub gain: f32,
}

impl MockMultirotorZ {
    pub const fn new(hover_throttle: f32) -> Self {
        const G: f32 = 9.81;
        let mass: f32 = 1.0;
        let gain = mass * G / hover_throttle;
        Self { altitude: 0.0, vertical_speed: 0.0, hover_throttle, mass, gain }
    }

    // Simulates physics movement over a tiny slice of time
    pub fn step(&mut self, throttle: f32, dt: f32) {
        let motor_force = throttle - self.hover_throttle;
        let drag_force = 0.1 * self.vertical_speed;

        let acceleration = (motor_force - drag_force) / self.mass;

        // Update velocity_speed and altitude using Euler integration
        self.vertical_speed += acceleration * dt;
        self.altitude += self.vertical_speed * dt;
    }
}

pub struct MockMultirotorXY {
    pub position_earth: Vector2df32,
    pub velocity_earth: Vector2df32,
    pub drag_coefficient: f32,
}

impl MockMultirotorXY {
    pub const fn new() -> Self {
        Self {
            position_earth: Vector2df32::new(0.0, 0.0),
            velocity_earth: Vector2df32::new(0.0, 0.0),
            drag_coefficient: 0.4,
        }
    }

    pub fn step(&mut self, target_angles: RollPitchf32, orientation: Quaternionf32, delta_time: f32) {
        let cos_yaw = orientation.cos_yaw();
        let sin_yaw = orientation.sin_yaw();

        let body_force_x = -target_angles.pitch;
        let body_force_y = target_angles.roll;

        let acceleration_north =
            (body_force_x * cos_yaw - body_force_y * sin_yaw) - (self.drag_coefficient * self.velocity_earth.x);
        let acceleration_east =
            (body_force_x * sin_yaw + body_force_y * cos_yaw) - (self.drag_coefficient * self.velocity_earth.y);

        self.velocity_earth.x += acceleration_north * delta_time;
        self.velocity_earth.y += acceleration_east * delta_time;
        self.position_earth += self.velocity_earth * delta_time;
    }
}
