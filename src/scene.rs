use bevy::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub albedo: Vec3,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            radius: 0.5,
            albedo: Vec3::ONE,
        }
    }
}
