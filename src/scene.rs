use bevy::{math::vec3, prelude::*, render::primitives::Aabb};

#[derive(Debug, Default, Clone, Resource)]
pub struct Scene {
    pub sky: Sky,
    pub materials: Vec<Material>,
    pub spheres: Vec<Sphere>,
    pub meshes: Vec<TriangleMesh>,
    pub lights: Vec<Light>,
}

#[derive(Debug, Clone, Copy)]
pub struct Sky {
    pub ground_color: Vec3,
    pub horizon_color: Vec3,
    pub zenith_color: Vec3,
    // pub sun_focus: f32,
    // pub sun_intensity: f32,
    // pub sun_direction: Vec3,
}

impl Sky {
    #[allow(unused)]
    pub const BLACK: Self = Self {
        zenith_color: Vec3::ZERO,
        horizon_color: Vec3::ZERO,
        ground_color: Vec3::ZERO,
        // sun_focus: 1.0,
        // sun_intensity: 1.0,
        // sun_direction: Vec3::ONE,
    };
}
impl Default for Sky {
    fn default() -> Self {
        Self {
            ground_color: vec3(0.2, 0.2, 0.2),
            horizon_color: Vec3::ONE,
            zenith_color: Vec3::ZERO,
            // sun_focus: 1.0,
            // sun_intensity: 1.0,
            // sun_direction: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub albedo: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub specular: f32,
    pub emissive_color: Vec3,
    pub emissive_power: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::ONE,
            roughness: 1.0,
            metallic: 0.0,
            specular: -1.0,
            emissive_color: Vec3::ZERO,
            emissive_power: 0.0,
        }
    }
}

impl Material {
    pub fn get_emission(&self) -> Vec3 {
        self.emissive_color * self.emissive_power
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

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material_id: usize,
    pub aabb: Aabb,
}

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub direction: Vec3,
    pub intensity: f32,
}
