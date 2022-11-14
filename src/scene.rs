use bevy::prelude::*;

#[derive(Debug, Default, Clone, Resource)]
pub struct Scene {
    pub sky_color: Vec3,
    pub materials: Vec<Material>,
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub albedo: Vec3,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::ONE,
            roughness: 1.0,
            metallic: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub material_id: usize,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            radius: 0.5,
            material_id: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub direction: Vec3,
    pub intensity: f32,
}
