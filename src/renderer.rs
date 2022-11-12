use bevy::prelude::*;

use crate::camera::ChernoCamera;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
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

    pub fn render(&mut self, camera: &ChernoCamera) {
        let mut ray = Ray {
            origin: camera.position,
            direction: Vec3::ZERO,
        };

        for y in 0..self.height {
            for x in 0..self.width {
                ray.direction = camera.ray_directions[x + y * self.width];
                let color = trace_ray(ray).clamp(Vec4::ZERO, Vec4::ONE);
                self.image_data[x + y * self.width] = color.as_u8_array();
            }
        }
    }
}

fn trace_ray(ray: Ray) -> Vec4 {
    let radius = 0.5;

    // (bx^2 + by^2)t^2 + (2(axbx + ayby))t + (ax^2 + ay^2 - r^2) = 0
    // where
    // a = ray origin
    // b = ray direction
    // r = radius
    // t = hit distance

    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * ray.origin.dot(ray.direction);
    let c = ray.origin.dot(ray.origin) - radius * radius;

    // Quadratic forumula discriminant:
    // b^2 - 4ac

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return Vec4::new(0.0, 0.0, 0.0, 1.0);
    }

    // Quadratic formula:
    // (-b +- sqrt(discriminant)) / 2a

    let closest_t = (-b - discriminant.sqrt()) / (2.0 * a);
    let _t0 = (-b + discriminant.sqrt()) / (2.0 * a);

    let hit_point = ray.origin + ray.direction * closest_t;
    let normal = hit_point.normalize();

    let light_dir = Vec3::new(1.0, 1.0, 1.0).normalize();
    let light_intensity = normal.dot(light_dir).max(0.0); // == cos(angle)

    let mut sphere_color = Vec3::new(1.0, 0.0, 1.0);
    sphere_color *= light_intensity;

    sphere_color.extend(1.0)
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
