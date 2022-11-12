use bevy::prelude::*;

#[derive(Debug)]
pub struct Renderer {
    pub image_data: Vec<u32>,
    pub width: usize,
    pub height: usize,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            image_data: vec![0; width * height],
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        *self = Self::new(width, height);
    }

    pub fn render(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let coord = Vec2::new(x as f32 / self.width as f32, y as f32 / self.height as f32);
                let coord = coord * 2.0 - 1.0;
                let color: Color = per_pixel(coord).clamp(Vec4::ZERO, Vec4::ONE).into();
                self.image_data[x + y * self.width] = color.as_rgba_u32();
            }
        }
    }
}

fn per_pixel(coord: Vec2) -> Vec4 {
    let ray_origin = Vec3::new(0.0, 0.0, 2.0);
    let ray_direction = Vec3::new(coord.x, coord.y, -1.0);
    let radius = 0.5;

    // (bx^2 + by^2)t^2 + (2(axbx + ayby))t + (ax^2 + ay^2 - r^2) = 0
    // where
    // a = ray origin
    // b = ray direction
    // r = radius
    // t = hit distance

    let a = ray_direction.dot(ray_direction);
    let b = 2.0 * ray_origin.dot(ray_direction);
    let c = ray_origin.dot(ray_origin) - radius * radius;

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

    let hit_point = ray_origin + ray_direction * closest_t;
    let normal = hit_point.normalize();

    let light_dir = Vec3::new(1.0, -1.0, 1.0).normalize();
    let light_intensity = normal.dot(light_dir).max(0.0); // == cos(angle)

    let mut sphere_color = Vec3::new(1.0, 0.0, 1.0);
    sphere_color *= light_intensity;

    sphere_color.extend(1.0)
}
