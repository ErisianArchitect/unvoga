// A 16x16 chunk of noise values.

use noise::OpenSimplex;
use rand::prelude::*;
use sha2::{Sha256, Digest};

#[cfg(test)]
mod testing_sandbox {
    use sha2::{Sha256, digest::Update, Digest};

    // TODO: Remove this sandbox when it is no longer in use.
    use super::*;
    #[test]
    fn sandbox() {
        let mut hasher = Sha256::default();
        Digest::update(&mut hasher, b"Hello, world");
        let result = hasher.finalize();
        let mut buffer = [0u8; 32];
        buffer.copy_from_slice(&result);
        let mut rng = rand::rngs::StdRng::from_seed(buffer);
        println!("Seed1: {}", rng.next_u64());
        println!("Seed2: {}", rng.next_u64());
        println!("Seed3: {}", rng.next_u64());
        println!("Seed4: {}", rng.next_u64());
    }
}

pub struct Curve {
    knots: Box<[f32]>,
    
}

pub struct NoiseGenerator {
    temperature_gen: OpenSimplex,
    humidity_gen: OpenSimplex,
    continentalness_gen: OpenSimplex,
    peaks_and_valleys_gen: OpenSimplex,
    civilization_gen: OpenSimplex,
    ruins_gen: OpenSimplex,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoiseData {
    pub temperature: f32,
    pub humidity: f32,
    pub continentalness: f32,
    pub peaks_and_valleys: f32,
    pub civilization: f32,
    pub ruins: f32,
}

impl NoiseData {
    pub const fn new(
        temperature: f32,
        humidity: f32,
        continentalness: f32,
        peaks_and_valleys: f32,
        civilization: f32,
        ruins: f32
    ) -> Self {
        Self {
            temperature,
            humidity,
            continentalness,
            peaks_and_valleys,
            civilization,
            ruins,
        }
    }

    pub const fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub const fn humidity(mut self, humidity: f32) -> Self {
        self.humidity = humidity;
        self
    }
    
    pub const fn continentalness(mut self, continentalness: f32) -> Self {
        self.continentalness = continentalness;
        self
    }

    pub const fn civilization(mut self, civilization: f32) -> Self {
        self.civilization = civilization;
        self
    }
    
    pub const fn ruins(mut self, ruins: f32) -> Self {
        self.ruins = ruins;
        self
    }
}

/// A plane of 16x16 [NoiseData].
#[derive(Debug, Clone)]
pub struct NoisePlane {
    cells: Box<[NoiseData]>,
}

impl NoisePlane {
    pub fn new() -> Self {
        Self {
            cells: (0..256).map(|_| NoiseData::default()).collect()
        }
    }

    // pub fn generate(chunk_x: i32, chunk_y: i32) -> Self {
    //     Self {
    //         cells: 
    //     }
    // }
}

impl Default for NoisePlane {
    fn default() -> Self {
        Self::new()
    }
}

fn deindex16x16(index: usize) -> (usize, usize) {
    (
        index & 0xf,
        index >> 4 & 0xf
    )
}

/// Ordered in yzx order (where x is the least significant and y is the most significant).
fn deindex16x16x16(index: usize) -> (usize, usize, usize) {
    (
        index & 0xf,
        index >> 8 & 0xf,
        index >> 4 & 0xf,
    )
}