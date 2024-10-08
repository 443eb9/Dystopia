use std::ops::Range;

use bevy::math::DVec2;
use rand::Rng;
use rand_distr::{num_traits::Float, Distribution, Normal, StandardNormal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangularDirection {
    Left,
    Right,
    Down,
}

pub fn polar_to_cartesian(theta: f64, r: f64) -> DVec2 {
    let (s, c) = theta.sin_cos();
    DVec2::new(c * r, s * r)
}

pub fn sample_normal_bounded<T>(rng: &mut impl Rng, distr: Normal<T>, min_max: [T; 2]) -> T
where
    T: Float,
    StandardNormal: Distribution<T>,
{
    loop {
        let t = rng.sample(distr);
        if t > min_max[0] && t < min_max[1] {
            return t;
        }
    }
}

pub fn reject_sampling(
    rng: &mut impl Rng,
    pdf: impl Fn(f64) -> f64,
    x_range: Range<f64>,
    y_range: Range<f64>,
    num_samples: u32,
    batch_size: u32,
) -> Vec<f64> {
    let num_samples = num_samples as usize;
    let mut samples = Vec::with_capacity(num_samples as usize);

    while samples.len() < num_samples {
        let x = (0..batch_size)
            .map(|_| rng.gen_range(x_range.clone()))
            .collect::<Vec<_>>();
        let y = (0..batch_size)
            .map(|_| rng.gen_range(y_range.clone()))
            .collect::<Vec<_>>();

        let new_samples = x
            .into_iter()
            .zip(y)
            .filter_map(|(x, y)| if y < pdf(x) { Some(x) } else { None })
            .collect::<Vec<_>>();

        if samples.len() + new_samples.len() < num_samples {
            samples.extend(new_samples);
        } else {
            samples.extend(new_samples.into_iter().take(num_samples - samples.len()));
            break;
        }
    }

    samples
}
