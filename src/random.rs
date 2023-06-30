use bevy::{math::Vec3A, prelude::Vec3};

#[allow(unused)]
pub fn wang_hash(mut seed: u32) -> u32 {
    seed = (seed ^ 61) ^ (seed >> 16);
    seed *= 9;
    seed ^= seed >> 4;
    seed *= 0x27d4eb2d;
    seed ^= seed >> 15;
    seed
}

pub fn pcg_hash(input: u32) -> u32 {
    let state = input.wrapping_mul(747796405).wrapping_add(2891336453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277803737);
    (word >> 22) ^ word
}

pub fn float(seed: &mut u32) -> f32 {
    *seed = pcg_hash(*seed);
    *seed as f32 / u32::MAX as f32
}

pub fn in_unit_sphere(seed: &mut u32) -> Vec3A {
    Vec3A::new(
        float(seed) * 2.0 - 1.0,
        float(seed) * 2.0 - 1.0,
        float(seed) * 2.0 - 1.0,
    )
}

#[cfg(test)]
mod test {
    use super::{float, pcg_hash};

    #[test]
    fn test_random() {
        let mut seed = 42;
        println!("pcg_hash: {} seed: {seed}", pcg_hash(seed));

        println!("float: {} hash: {}", float(&mut seed), seed);
        println!("float: {} hash: {}", float(&mut seed), seed);
        println!("float: {} hash: {}", float(&mut seed), seed);
        println!("{}", seed);
    }
}
