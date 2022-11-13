use bevy::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct Scene {
    pub materials: Vec<Material>,
    pub spheres: Vec<Sphere>,
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
