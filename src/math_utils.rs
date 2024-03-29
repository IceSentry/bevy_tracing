use bevy::math::Vec3A;

// For the incident vector I and surface orientation N, returns the reflection direction
#[allow(unused)]
pub fn reflect(i: Vec3A, n: Vec3A) -> Vec3A {
    i - 2.0 * n.dot(i) * n
}

pub fn smoothstep(edge0: f32, edge1: f32, t: f32) -> f32 {
    if t < edge0 {
        return 0.0;
    }
    if t >= edge1 {
        return 1.0;
    }
    let t = (t - edge0) / (edge1 - edge0);
    t * t * (3.0 - 2.0 * t)
}
