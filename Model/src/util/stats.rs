use rand::distributions::uniform::SampleUniform;
use rand::distributions::Uniform;
use rand::rngs::SmallRng;
use rand::Rng;
use statrs::distribution::Normal;

use crate::util::{BaseFloat, UtilError};

/// Pick from a uniform distribution
pub fn uni_dist<T>(bounds: (T, T), seed: &mut SmallRng) -> i64
where
    T: Into<i64> + SampleUniform,
{
    let uniform = Uniform::new(bounds.0.into(), bounds.1.into());
    seed.sample(uniform)
}

/// Pick from a normal distribution
pub fn norm_dist(center: &BaseFloat, std_dev: &BaseFloat, seed: &mut SmallRng) -> Result<BaseFloat, UtilError> {
    let normal = Normal::new((*center).into(), (*std_dev).into())?;
    Ok(seed.sample(normal) as BaseFloat)
}

/// Pick from a normal distribution
pub fn random_percentage(seed: &mut SmallRng) -> BaseFloat { seed.gen_range(0.0..100.0) / 100.0 }
