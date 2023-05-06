use crate::prelude::*;

use std::f32::consts::PI;

pub struct Camera {
    front: Vec3,
    right: Vec3,
    up: Vec3,

    yaw: f32,
    pitch: f32,

    position: Vec3,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            front: Vec3::new(0.0, 0.0, -1.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            yaw: -(PI / 2.0),
            pitch: 0.0,
            position,
        }
    }

    pub fn move_forward(&mut self, delta: f32) {
        self.position += self.front * delta;
    }

    pub fn move_backward(&mut self, delta: f32) {
        self.position -= self.front * delta;
    }

    pub fn move_left(&mut self, delta: f32) {
        self.position -= self.right * delta;
    }

    pub fn move_right(&mut self, delta: f32) {
        self.position += self.right * delta;
    }

    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta;

        self.pitch += pitch_delta;
        let pitch_constrain = (PI / 2.0) - 0.001;
        if self.pitch > pitch_constrain {
            self.pitch = pitch_constrain;
        }
        if self.pitch < -pitch_constrain {
            self.pitch = -pitch_constrain;
        }

        self.front = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        let world_up = Vec3::new(0.0, 1.0, 0.0);
        self.right = self.front.cross(world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn view_mat(&self) -> Mat4 {
        let center = self.position + self.front;
        Mat4::look_at_rh(self.position, center, self.up)
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }
}
