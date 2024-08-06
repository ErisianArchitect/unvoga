#![allow(unused)]

use std::path::Path;
use itertools::Itertools;
use noise::{NoiseFn, OpenSimplex};
use rand::{RngCore, SeedableRng};
use sha2::{Digest, Sha256};

// fn octave_noise(noise_fn: &OpenSimplex, point: Point, octaves: u32, persistence: f64, lacunarity: f64, scale: f64, initial_amplitude: f64) -> f64 {
//     let mut total = 0.0;
//     let mut frequency = scale;
//     let mut amplitude = initial_amplitude;
//     let mut max_value = 1.0;
//     let mut scale = 1.0;
//     for _ in 0..octaves {
//         let noise_value = noise_fn.get([point.x * frequency, point.y * frequency]);
//         total += scale * noise_value * amplitude;
//         scale *= 0.5;
//         max_value += amplitude;
//         frequency *= lacunarity;
//         amplitude *= persistence;
//     }
//     // total / max_value
//     total
// }

fn seed_rng<B: AsRef<[u8]>>(bytes: B) -> rand::rngs::StdRng {
    let mut hasher = Sha256::default();
    Digest::update(&mut hasher, bytes.as_ref());
    let result = hasher.finalize();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&result);
    rand::rngs::StdRng::from_seed(seed)
}

fn make_seed<B: AsRef<[u8]>>(bytes: B) -> u32 {
    let mut rng = seed_rng(bytes);
    rng.next_u32()
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct SimplexInterval<'a> {
    simplex: &'a OpenSimplex,
    interval: &'a NoiseGenIntervalSampler,
}

/// Layered Open Simplex noise.
#[derive(Debug, Clone)]
pub struct NoiseGen {
    simplexes: Vec<SimplexSampler>,
    weights: Vec<f64>,
    octave_gen: OctaveGen,
}

#[derive(Debug, Clone)]
pub struct SimplexSampler {
    simplex: OpenSimplex,
    intervals: Vec<NoiseGenIntervalSampler>,
    weights: Vec<f64>,
    octave_gen: OctaveGen,
}

#[derive(Debug, Clone)]
pub struct NoiseGenIntervalSampler {
    spline: Option<splines::Spline<f64, f64>>,
    octaves: u32,
    octave_gen: OctaveGen,
    invert: bool,
    bounds: NoiseBounds,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoiseGenConfig {
    simplexes: Vec<SimplexConfig>,
    octave_gen: OctaveGen,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimplexConfig {
    enabled: bool,
    intervals: Vec<NoiseGenIntervalConfig>,
    octave_gen: OctaveGen,
    seed: String,
    weight: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoiseGenIntervalConfig {
    enabled: bool,
    spline: SplineConfig,
    octaves: u32,
    octave_gen: OctaveGen,
    invert: bool,
    bounds: NoiseBounds,
    weight: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SplineConfig {
    enabled: bool,
    spline: Vec<InterpKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
struct InterpKey {
    x: f64,
    y: f64,
    interpolation: Interpolation,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
enum Interpolation {
    CatmullRom = 0,
    Cosine = 1,
    Linear = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OctaveBlend {
    Scale = 0,
    Multiply = 1,
    Average = 2,
    Min = 3,
    Max = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OctaveGen {
    pub persistence: f64,
    pub lacunarity: f64,
    pub initial_amplitude: f64,
    pub scale: f64,
    pub x_mult: f64,
    pub y_mult: f64,
    pub rotation: f64,
    pub blend_mode: OctaveBlend,
    pub offset: (f64, f64),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub enum NoiseBoundMode {
    Clamp,
    Cutoff,
    Range,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoiseBound {
    pub t: f64,
    pub mode: NoiseBoundMode,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoiseBounds {
    pub low: NoiseBound,
    pub high: NoiseBound,
}

impl NoiseGenConfig {
    pub fn export<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<bincode::ErrorKind>> {
        let data = bincode::serialize(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn import<P: AsRef<Path>>(path: P) -> Result<Self, Box<bincode::ErrorKind>> {
        let data = std::fs::read(path)?;
        bincode::deserialize(&data)
    }
}

struct NoiseLayer {
    noise: f64,
    amplitude: f64,
    frequency: f64,
    total_amplitude: f64,
}

impl NoiseLayer {
    pub const fn new(amplitude: f64, frequency: f64, noise: f64) -> Self {
        Self {
            amplitude,
            frequency,
            noise,
            total_amplitude: 0.0,
        }
    }
}

impl OctaveGen {
    pub fn weighted_sample<F: NoiseSampler, It: IntoIterator<Item = F>>(&self, point: Point, weights: &[f64], layers: It) -> f64 {
        let mut scale = 1.0;
        let angle = self.rotation.to_radians();
        let point = Point::new(point.x + self.offset.0, point.y + self.offset.1);
        let point = if self.rotation != 0.0 {
            Point::new(
                point.x * angle.cos() - point.y * angle.sin(),
                point.x * angle.sin() + point.y * angle.cos()
            )
        } else {
            point
        };
        let point = Point::new(point.x * self.x_mult, point.y * self.y_mult);
        match self.blend_mode {
            OctaveBlend::Scale => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    0.0,  
                );
                layers.into_iter().enumerate().fold(init, |mut accum, (i, noise)| {
                    accum.noise += scale * noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude * weights[i];
                    scale *= 0.5;
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                }).noise
            },
            OctaveBlend::Multiply => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    1.0,  
                );
                layers.into_iter().enumerate().fold(init, |mut accum, (i, noise)| {
                    accum.noise *= noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude * weights[i];
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                }).noise
            },
            OctaveBlend::Average => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    0.0,  
                );
                let result = layers.into_iter().enumerate().fold(init, |mut accum, (i, noise)| {
                    accum.noise += noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude * weights[i];
                    accum.total_amplitude += accum.amplitude;
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                });
                result.noise / result.total_amplitude
            }
            OctaveBlend::Min => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    1.0,  
                );
                let result = layers.into_iter().enumerate().fold(init, |mut accum, (i, noise)| {
                    accum.noise = accum.noise.min(noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude * weights[i]);
                    accum.total_amplitude += accum.amplitude;
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                });
                result.noise
            }
            OctaveBlend::Max => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    0.0,  
                );
                let result = layers.into_iter().enumerate().fold(init, |mut accum, (i, noise)| {
                    accum.noise = accum.noise.max(noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude * weights[i]);
                    accum.total_amplitude += accum.amplitude;
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                });
                result.noise
            }
        }
    }
    
    pub fn sample<F: NoiseSampler, It: IntoIterator<Item = F>>(&self, point: Point, layers: It) -> f64 {
        let mut scale = 1.0;
        let point = Point::new(point.x + self.offset.0, point.y + self.offset.1);
        let point = if self.rotation != 0.0 {
            let angle = self.rotation.to_radians();
            Point::new(
                point.x * angle.cos() - point.y * angle.sin(),
                point.x * angle.sin() + point.y * angle.cos()
            )
        } else {
            point
        };
        let point = Point::new(point.x * self.x_mult, point.y * self.y_mult);
        match self.blend_mode {
            OctaveBlend::Scale => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    0.0,  
                );
                layers.into_iter().fold(init, |mut accum, noise| {
                    accum.noise += scale * noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude;
                    scale *= 0.5;
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                }).noise
            },
            OctaveBlend::Multiply => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    1.0,  
                );
                layers.into_iter().fold(init, |mut accum, noise| {
                    accum.noise *= noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude;
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                }).noise
            },
            OctaveBlend::Average => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    0.0,  
                );
                let result = layers.into_iter().fold(init, |mut accum, noise| {
                    accum.noise += noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude;
                    accum.total_amplitude += accum.amplitude;
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                });
                result.noise / result.total_amplitude
            }
            OctaveBlend::Min => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    1.0,  
                );
                let result = layers.into_iter().fold(init, |mut accum, noise| {
                    accum.noise = accum.noise.min(noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude);
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                });
                result.noise
            }
            OctaveBlend::Max => {
                let init = NoiseLayer::new(
                    self.initial_amplitude,
                    self.scale,
                    0.0,  
                );
                let result = layers.into_iter().fold(init, |mut accum, noise| {
                    accum.noise = accum.noise.max(noise.sample_noise(Point::new(point.x * accum.frequency, point.y * accum.frequency)) * accum.amplitude);
                    accum.amplitude *= self.persistence;
                    accum.frequency *= self.lacunarity;
                    accum
                });
                result.noise
            }
        }
    }
}

pub trait NoiseSampler {
    fn sample_noise(self, point: Point) -> f64;
}

impl SimplexSampler {
    pub fn enabled(&self) -> bool {
        !self.intervals.is_empty()
    }

    pub fn sample(&self, point: Point) -> f64 {
        self.octave_gen.weighted_sample(point, &self.weights ,self.intervals.iter().map(|interval| SimplexInterval {
            interval,
            simplex: &self.simplex
        }))
    }
}

impl NoiseSampler for &SimplexSampler {
    fn sample_noise(self, point: Point) -> f64 {
        self.sample(point)
    }
}

impl NoiseGen {
    pub fn sample(&self, point: Point) -> f64 {
        if self.simplexes.is_empty() {
            return 0.0;
        }
        self.octave_gen.weighted_sample(point, &self.weights, self.simplexes.iter())
    }
}

impl<'a> NoiseSampler for SimplexInterval<'a> {
    fn sample_noise(self, point: Point) -> f64 {
        self.interval.sample(&self.simplex, point)
    }
}

impl<F: FnMut(Point) -> f64> NoiseSampler for F {
    fn sample_noise(mut self, point: Point) -> f64 {
        let mut f = self;
        f(point)
    }
}

impl NoiseSampler for &OpenSimplex {
    fn sample_noise(self, point: Point) -> f64 {
        self.get([point.x, point.y])
    }
}

impl NoiseGenIntervalSampler {
    pub fn sample(&self, simplex: &OpenSimplex, point: Point) -> f64 {
        let point = Point::new(point.x * 0.125, point.y * 0.125);
        let noise = self.octave_gen.sample(point, (0..self.octaves).map(|_| simplex));
        let gradient = (noise + 1.) * 0.5;
        let gradient = if let Some(spline) = &self.spline {
            spline.sample(gradient).unwrap_or(gradient)
        } else {
            gradient
        };
        let gradient = self.bounds.bound(gradient);
        if self.invert {
            -gradient + 1.0
        } else {
            gradient
        }
    }
}

impl NoiseBounds {
    pub const fn new(low: NoiseBound, high: NoiseBound) -> Self {
        Self {
            low,
            high
        }
    }

    pub fn bound(self, t: f64) -> f64 {
        use NoiseBoundMode::*;
        match (self.low.mode, self.high.mode) {
            (Range, Range) => {
                let low = self.low.t;
                let high = self.high.t;
                let clamped = t.max(low).min(high);
                let diff = high - low;
                let rel = clamped - low;
                rel * (1. / diff)
            }
            (Range, Clamp) => {
                let low = self.low.t;
                let clamped = t.max(low).min(self.high.t);
                let diff = 1.0 - low;
                let rel = clamped - low;
                rel * (1. / diff)
            }
            (Range, Cutoff) => {
                if t > self.high.t {
                    0.0
                } else {
                    let low = self.low.t;
                    let clamped = t.max(low);
                    let diff = 1.0 - low;
                    let rel = clamped - low;
                    rel * (1. / diff)
                }
            }
            (Clamp, Range) => {
                let high = self.high.t;
                let clamped = t.max(self.low.t).min(high);
                clamped * (1. / high)
            }
            (Clamp, Clamp) => {
                t.max(self.low.t).min(self.high.t)
            }
            (Clamp, Cutoff) => {
                if t > self.high.t {
                    0.
                } else {
                    t.max(self.low.t)
                }
            }
            (Cutoff, Range) => {
                if t < self.low.t {
                    0.
                } else {
                    let high = self.high.t;
                    let clamped = t.min(high);
                    clamped * (1. / high)
                }
            }
            (Cutoff, Clamp) => {
                if t < self.low.t {
                    0.
                } else {
                    t.min(self.high.t)
                }
            }
            (Cutoff, Cutoff) => {
                if t < self.low.t || t > self.high.t {
                    0.
                } else {
                    t
                }
            }
        }
    }
}

impl Into<splines::Key<f64, f64>> for InterpKey {
    fn into(self) -> splines::Key<f64, f64> {
        splines::Key { t: self.x, value: self.y, interpolation: match self.interpolation {
            Interpolation::CatmullRom => splines::Interpolation::CatmullRom,
            Interpolation::Cosine => splines::Interpolation::Cosine,
            Interpolation::Linear => splines::Interpolation::Linear,
        }}
    }
}

impl From<NoiseGenIntervalConfig> for NoiseGenIntervalSampler {
    fn from(value: NoiseGenIntervalConfig) -> Self {
        Self {
            spline: if value.spline.enabled {
                Some(
                    splines::Spline::from_iter(value.spline.spline.iter().cloned().map(|key| key.into()))
                )
            } else {
                None
            },
            octaves: value.octaves,
            octave_gen: value.octave_gen,
            invert: value.invert,
            bounds: value.bounds,
        }
    }
}

impl SimplexSampler {
    pub fn from_config(config: SimplexConfig, seed: u32) -> Self {
        let mut total_weight = 0.0;
        let mut weights = config.intervals.iter().map(|interval| {
            total_weight += interval.weight;
            interval.weight
        }).collect_vec();
        let weight_mul = 1.0 / total_weight;
        weights.iter_mut().for_each(|weight| *weight *= weight_mul);
        Self {
            weights,
            intervals: config.intervals.into_iter().filter_map(|interval| {
                if interval.enabled {
                    Some(interval.into())
                } else {
                    None
                }
            }).collect(),
            octave_gen: config.octave_gen,
            simplex: OpenSimplex::new(seed),
        }
    }
}

impl NoiseGen {
    pub fn from_config<B: AsRef<[u8]>>(config: NoiseGenConfig, seed: B) -> Self {
        let mut rng = seed_rng(seed);
        let mut total_weight = 0.0;
        let mut weights = config.simplexes.iter().map(|simp| {
            total_weight += simp.weight;
            simp.weight
        }).collect_vec();
        let weight_mul = 1.0 / total_weight;
        weights.iter_mut().for_each(|weight| *weight *= weight_mul);
        Self {
            weights,
            octave_gen: config.octave_gen,
            simplexes: config.simplexes.into_iter().filter_map(|simp| {
                if simp.enabled && !simp.intervals.is_empty() {
                    Some(SimplexSampler::from_config(simp, rng.next_u32()))
                } else {
                    None
                }
            }).collect()
        }
    }
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Point {
        Self {
            x,
            y
        }
    }

    pub fn dot(self, other: Point) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn normalized(self) -> Self {
        let magnitude = self.magnitude();
        Self::new(self.x / magnitude, self.y / magnitude)
    }

    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn check_between(self, min: Self, max: Self) -> Option<f64> {
        let ab = max - min;
        let ap = self - min;
        let ab_dot_ab = ab.dot(ab);
        let ap_dot_ab = ap.dot(ab);
        let t = ap_dot_ab / ab_dot_ab;
        if 0.0 <= t && t <= 1.0 {
            Some(t)
        } else {
            None
        }
    }

    pub fn distance_between(self, a: Point, b: Point) -> f64 {
        let ray = (b - a).normalized();
        let point = self - a;
        point.dot(ray)
    }

    pub fn check_between_closest(self, min: Self, max: Self) -> Option<Self> {
        let ab = max - min;
        let ap = self - min;
        let ab_dot_ab = ab.dot(ab);
        let ap_dot_ab = ap.dot(ab);
        let t = ap_dot_ab / ab_dot_ab;
        if 0.0 <= t && t <= 1.0 {
            Some(min + t * ab)
        } else {
            None
        }
    }
}

impl std::ops::Add<Point> for Point {
    type Output = Self;
    fn add(self, rhs: Point) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Mul<Point> for f64 {
    type Output = Point;
    fn mul(self, rhs: Point) -> Self::Output {
        Point::new(self * rhs.x, self * rhs.y)
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Self;
    fn sub(self, rhs: Point) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

// trait BoolUiExt: Sized + Copy {

//     fn toggle(&mut self) -> bool;

//     fn opt<T>(self, value: T) -> Option<T>;
//     fn not_opt<T>(self, value: T) -> Option<T>;
//     fn opt_with<T, F: FnMut() -> T>(self, f: F) -> Option<T>;
//     fn not_opt_with<T, F: FnMut() -> T>(self, f: F) -> Option<T>;

//     fn select<T>(self, _true: T, _false: T) -> T;
// }

// impl BoolUiExt for bool {
//     fn toggle(&mut self) -> bool {
//         let old = *self;
//         *self = !old;
//         old
//     }
    
//     fn select<T>(self, _true: T, _false: T) -> T {
//         if self {
//             _true
//         } else {
//             _false
//         }
//     }

//     fn not_opt<T>(self, value: T) -> Option<T> {
//         if self {
//             None
//         } else {
//             Some(value)
//         }
//     }

//     fn opt<T>(self, value: T) -> Option<T> {
//         if self {
//             Some(value)
//         } else {
//             None
//         }
//     }

//     fn not_opt_with<T, F: FnMut() -> T>(self, mut f: F) -> Option<T> {
//         if self {
//             None
//         } else {
//             Some(f())
//         }
//     }

//     fn opt_with<T, F: FnMut() -> T>(self, mut f: F) -> Option<T> {
//         if self {
//             Some(f())
//         } else {
//             None
//         }
//     }
// }

#[cfg(test)]
mod testing_sandbox {
    // TODO: Remove this sandbox when it is no longer in use.
    use super::*;
    #[test]
    fn sandbox() {
        let config = NoiseGenConfig::import("test.simp").expect("Failed to import");
        let sampler = NoiseGen::from_config(config, "Hello, world!");
        println!("Sample: {}", sampler.sample(Point::new(0., 0.)));
        // println!("{}", config.simplexes.last().unwrap().seed)
    }
}