use bevy::{
    math::{vec4, Vec4Swizzles},
    prelude::*,
};

use crate::{
    camera::ChernoCamera,
    scene::{Scene, Sphere},
};

const CLEAR_COLOR: Vec4 = vec4(0.0, 0.0, 0.0, 1.0);

#[derive(Debug, Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

struct HitPayload {
    hit_distance: f32,
    world_position: Vec3,
    world_normal: Vec3,
    object_index: usize,
}

#[derive(Debug)]
pub struct Renderer {
    pub image_data: Vec<[u8; 4]>,
    pub width: usize,
    pub height: usize,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            image_data: vec![[0, 0, 0, 0]; width * height],
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.image_data.resize(width * height, [0, 0, 0, 0]);
        self.width = width;
        self.height = height;
    }

    pub fn render(&mut self, camera: &ChernoCamera, scene: &Scene, bounces: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel_index = x + y * self.width;
                let color =
                    per_pixel(scene, camera, pixel_index, bounces).clamp(Vec4::ZERO, Vec4::ONE);
                self.image_data[pixel_index] = color.as_u8_array();
            }
        }
    }
}

// For the incident vector I and surface orientation N, returns the reflection direction
fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - 2.0 * n.dot(i) * n
}

fn per_pixel(scene: &Scene, camera: &ChernoCamera, pixel_index: usize, bounces: u8) -> Vec4 {
    let mut ray = Ray {
        origin: camera.position,
        direction: camera.ray_directions[pixel_index],
    };
    let mut multiplier = 1.0;
    let mut color = Vec3::ZERO;
    for _ in 0..bounces {
        match trace_ray(&ray, scene) {
            None => {
                color += CLEAR_COLOR.xyz() * multiplier;
                break;
            }
            Some(payload) => {
                let light_dir = Vec3::new(1.0, 1.0, 1.0).normalize();
                let light_intensity = payload.world_normal.dot(light_dir).max(0.0); // == cos(angle)

                let sphere = scene.spheres[payload.object_index];
                let mut sphere_color = sphere.albedo;
                sphere_color *= light_intensity;

                color += sphere_color * multiplier;
                multiplier *= 0.7;

                ray.origin = payload.world_position + payload.world_normal * 0.0001;

                ray.direction = reflect(ray.direction, payload.world_normal);
            }
        }
    }
    color.extend(1.0)
}

fn closest_hit(scene: &Scene, ray: &Ray, hit_distance: f32, object_index: usize) -> HitPayload {
    let sphere = scene.spheres[object_index];
    let origin = ray.origin - sphere.position;
    let hit_position = origin + ray.direction * hit_distance;
    let world_normal = hit_position.normalize();

    HitPayload {
        hit_distance,
        object_index,
        world_position: hit_position + sphere.position,
        world_normal,
    }
}

fn trace_ray(ray: &Ray, scene: &Scene) -> Option<HitPayload> {
    let mut closest_object: Option<usize> = None;
    let mut hit_distance = f32::MAX;
    for (i, sphere) in scene.spheres.iter().enumerate() {
        let origin = ray.origin - sphere.position;

        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * origin.dot(ray.direction);
        let c = origin.dot(origin) - sphere.radius * sphere.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            continue;
        }

        let closest_t = (-b - discriminant.sqrt()) / (2.0 * a);
        // let _t0 = (-b + discriminant.sqrt()) / (2.0 * a);
        if closest_t > 0.0 && closest_t < hit_distance {
            hit_distance = closest_t;
            closest_object = Some(i);
        }
    }
    closest_object.map(|object_index| closest_hit(scene, ray, hit_distance, object_index))
}

trait Vec4Ext {
    fn as_rgba_u32(&self) -> u32;

    fn as_u8_array(&self) -> [u8; 4];
}

impl Vec4Ext for Vec4 {
    fn as_rgba_u32(&self) -> u32 {
        u32::from_le_bytes(self.as_u8_array())
    }

    fn as_u8_array(&self) -> [u8; 4] {
        [
            (self.x * 255.0) as u8,
            (self.y * 255.0) as u8,
            (self.z * 255.0) as u8,
            (self.w * 255.0) as u8,
        ]
    }
}
