use bevy::{
    math::Vec3A,
    prelude::*,
    render::{mesh::Indices, primitives::Aabb},
};
use nanorand::Rng;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    camera::CustomCamera,
    math_utils::{reflect, smoothstep},
    scene::{Scene, Sphere},
};

#[derive(Debug, Clone, Copy)]
struct Ray {
    origin: Vec3A,
    direction: Vec3A,
    inv_direction: Vec3A,
}

struct HitPayload {
    #[allow(unused)]
    hit_distance: f32,
    world_position: Vec3,
    world_normal: Vec3,
    material_id: usize,
}

#[derive(Debug, Resource)]
pub struct Renderer {
    pub image_data: Vec<[u8; 4]>,
    pub accumulation_data: Vec<Vec4>,
    pub width: usize,
    pub height: usize,
    pub samples: usize,
    pub accumulate: bool,
    pub bounces: u8,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            image_data: vec![[0, 0, 0, 0]; width * height],
            accumulation_data: vec![Vec4::ZERO; width * height],
            width,
            height,
            samples: 1,
            accumulate: true,
            bounces: 5,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;

        self.image_data.resize(width * height, [0, 0, 0, 0]);
        self.accumulation_data.resize(width * height, Vec4::ZERO);

        self.reset_frame_index();
    }

    pub fn render(&mut self, camera: &CustomCamera, scene: &Scene) {
        if self.samples == 1 {
            self.accumulation_data.fill(Vec4::ZERO);
        }

        self.image_data
            .par_iter_mut()
            .zip(&mut self.accumulation_data)
            .enumerate()
            .for_each(|(pixel_index, (pixel, accumulated_pixel))| {
                let color = per_pixel(scene, camera, pixel_index, self.bounces);

                *accumulated_pixel += color;
                let mut accumulated_color = *accumulated_pixel;
                accumulated_color /= self.samples as f32;

                let color = accumulated_color.clamp(Vec4::ZERO, Vec4::ONE);
                *pixel = color.as_u8_array();
            });

        if self.accumulate {
            self.samples += 1;
        } else {
            self.samples = 1;
        }
    }

    pub fn reset_frame_index(&mut self) {
        self.samples = 1;
    }
}

fn sky_color(scene: &Scene, ray: &Ray) -> Vec3 {
    let sky_gradient_t = smoothstep(0.0, 0.4, ray.direction.y).powf(0.35);
    let sky_gradient = Vec3::lerp(
        scene.sky.horizon_color,
        scene.sky.zenith_color,
        sky_gradient_t,
    );
    // let sun = ray
    //     .direction
    //     .dot(scene.sky.sun_direction)
    //     .max(0.0)
    //     .powf(scene.sky.sun_focus)
    //     * scene.sky.sun_intensity;

    let ground_to_sky_t = smoothstep(-0.01, 0.0, ray.direction.y);
    // let sun_mask = (ground_to_sky_t >= 1.0) as i32 as f32;
    Vec3::lerp(scene.sky.ground_color, sky_gradient, ground_to_sky_t) // + sun * sun_mask
}

fn per_pixel(scene: &Scene, camera: &CustomCamera, pixel_index: usize, bounces: u8) -> Vec4 {
    let mut ray = Ray {
        origin: Vec3A::from(camera.position),
        direction: camera.ray_directions[pixel_index],
        inv_direction: 1.0 / camera.ray_directions[pixel_index],
    };
    let mut multiplier = 1.0;
    let mut color = Vec3::ZERO;
    let mut rng = nanorand::tls_rng();
    for _ in 0..bounces {
        match trace_ray(&ray, scene) {
            None => {
                color += sky_color(scene, &ray) * multiplier;
                break;
            }
            Some(payload) => {
                let material = scene.materials[payload.material_id];

                let mut hit_color = material.albedo;

                let mut light_intensity = 0.0;
                for light in &scene.lights {
                    let light_dir = light.direction.normalize();
                    light_intensity +=
                        payload.world_normal.dot(light_dir).max(0.0) * light.intensity;
                }

                hit_color *= light_intensity;

                color += hit_color * multiplier;
                multiplier *= 0.5;

                ray.origin = (payload.world_position + payload.world_normal * 0.0001).into();
                let rand_dir = Vec3::new(
                    rng.generate::<f32>(),
                    rng.generate::<f32>(),
                    rng.generate::<f32>(),
                ) - 0.5; // -0.5..0.5
                ray.direction = reflect(
                    ray.direction,
                    (payload.world_normal + material.roughness * rand_dir).into(),
                );
            }
        }
    }
    color.extend(1.0)
}

fn trace_ray(ray: &Ray, scene: &Scene) -> Option<HitPayload> {
    let mut sphere_hit_distance = f32::MAX;
    let mut triangle_hit_distance = f32::MAX;

    let mut closest_sphere: Option<usize> = None;
    for (i, sphere) in scene.spheres.iter().enumerate() {
        match sphere_intersect(ray, sphere) {
            None => continue,
            Some(closest_t) => {
                if closest_t > 0.0 && closest_t < sphere_hit_distance {
                    sphere_hit_distance = closest_t;
                    closest_sphere = Some(i);
                }
            }
        }
    }

    let mut normal = Vec3A::ZERO;
    let mut closest_mesh: Option<usize> = None;
    for (i, mesh) in scene.meshes.iter().enumerate() {
        if !aabb_intersect(ray, mesh.aabb) {
            continue;
        }

        let Some(Some(positions)) = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .map(|x| x.as_float3())
        else {
            panic!("Vertex positions attribute should exist and be float3");
        };
        let Some(Some(normals)) = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .map(|x| x.as_float3())
        else {
            panic!("Vertex normals attribute should exist and be float3");
        };

        let Some(Indices::U32(indices)) = mesh.mesh.indices() else {
            panic!("Only U32 indices are supported")
        };

        for indices in indices.chunks(3) {
            let [i0, i1, i2] = indices else { unreachable!() };

            match triangle_intersect(
                ray,
                positions[*i0 as usize].into(),
                positions[*i1 as usize].into(),
                positions[*i2 as usize].into(),
                normals[*i0 as usize].into(),
                normals[*i1 as usize].into(),
                normals[*i2 as usize].into(),
            ) {
                None => continue,
                Some((closest_t, hit_normal)) => {
                    if closest_t > 0.0 && closest_t < triangle_hit_distance {
                        triangle_hit_distance = closest_t;
                        normal = hit_normal;
                        closest_mesh = Some(i);
                    }
                }
            }
        }
    }

    if let Some(sphere_index) = closest_sphere {
        if sphere_hit_distance < triangle_hit_distance {
            let sphere = scene.spheres[sphere_index];
            let origin = Vec3::from(ray.origin) - sphere.position;
            let hit_position = origin + Vec3::from(ray.direction) * sphere_hit_distance;
            return Some(HitPayload {
                hit_distance: sphere_hit_distance,
                material_id: sphere.material_id,
                world_position: hit_position + sphere.position,
                world_normal: hit_position.normalize(),
            });
        }
    }

    if let Some(mesh_index) = closest_mesh {
        if triangle_hit_distance < sphere_hit_distance {
            let mesh = &scene.meshes[mesh_index];
            let translation = mesh.transform.translation;
            let origin = Vec3::from(ray.origin) - translation;
            let hit_position = origin + Vec3::from(ray.direction) * triangle_hit_distance;
            return Some(HitPayload {
                hit_distance: triangle_hit_distance,
                material_id: mesh.material_id,
                world_position: hit_position + translation,
                world_normal: normal.into(),
            });
        }
    }

    None
}

fn sphere_intersect(ray: &Ray, sphere: &Sphere) -> Option<f32> {
    let origin = ray.origin - Vec3A::from(sphere.position);

    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * origin.dot(ray.direction);
    let c = origin.dot(origin) - sphere.radius * sphere.radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let closest_t = (-b - discriminant.sqrt()) / (2.0 * a);
    // let _t0 = (-b + discriminant.sqrt()) / (2.0 * a);
    Some(closest_t)
}

// Scratch a pixel: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html
// Sebastian Lague: https://youtu.be/Qz0KTGYJtUk?t=1419
#[allow(non_snake_case)]
fn triangle_intersect(
    ray: &Ray,
    v0: Vec3A,
    v1: Vec3A,
    v2: Vec3A,
    n0: Vec3A,
    n1: Vec3A,
    n2: Vec3A,
) -> Option<(f32, Vec3A)> {
    let v0v1 = v1 - v0;
    let v0v2 = v2 - v0;
    let p_vec = ray.direction.cross(v0v2);
    let det = v0v1.dot(p_vec);

    if det < f32::EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;

    let t_vec = ray.origin - v0;
    let u = t_vec.dot(p_vec) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q_vec = t_vec.cross(v0v1);
    let v = ray.direction.dot(q_vec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = v0v2.dot(q_vec) * inv_det;
    let w = 1.0 - u - v;
    let N = (n0 * w + n1 * u + n2 * v).normalize();
    Some((t, N))
}

// https://tavianator.com/2022/ray_box_boundary.html
fn aabb_intersect(ray: &Ray, aabb: Aabb) -> bool {
    let mut tmin: f32 = 0.0;
    let mut tmax: f32 = f32::INFINITY;

    for i in 0..3 {
        let t1 = (Vec3::from(aabb.min())[i] - ray.origin[i]) * ray.inv_direction[i];
        let t2 = (Vec3::from(aabb.max())[i] - ray.origin[i]) * ray.inv_direction[i];

        tmin = t1.max(tmin).min(t2.max(tmin));
        tmax = t1.min(tmax).max(t2.min(tmax));
    }

    tmin < tmax
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
