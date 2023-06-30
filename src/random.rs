use bevy::math::Vec3A;
use rand::{Rng, RngCore};

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

pub struct PcgHashRng {
    pub seed: u32,
}

impl PcgHashRng {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }
}

impl RngCore for PcgHashRng {
    fn next_u32(&mut self) -> u32 {
        self.seed = pcg_hash(self.seed);
        self.seed
    }

    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_u32(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

pub fn in_unit_sphere<R: Rng>(rng: &mut R) -> Vec3A {
    Vec3A::new(
        rng.gen::<f32>() * 2.0 - 1.0,
        rng.gen::<f32>() * 2.0 - 1.0,
        rng.gen::<f32>() * 2.0 - 1.0,
    )
    .normalize()

    // let normal_distr = StandardNormal;
    // Vec3A::new(
    //     normal_distr.sample(rng),
    //     normal_distr.sample(rng),
    //     normal_distr.sample(rng),
    // )
    // .normalize()
}
