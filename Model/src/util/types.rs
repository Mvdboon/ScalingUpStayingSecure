//! All types that are used within the model are defined in this module.
use std::fmt::Debug;
use std::hash::Hash;

use derive_alias::derive_alias;
use derive_more::{Add, AddAssign, AsMut, AsRef, Display, Div, From, FromStr, Mul, Neg, Sub, SubAssign, Sum};
use serde::{Deserialize, Serialize};

/// All Traits that a Struct designating a type needs to fulfil.
pub trait StructTraitBound: Clone + Copy + Debug + PartialEq + PartialOrd + Eq + Ord + Hash + Serialize {}

derive_alias! {
    newtype =>  #[derive(Serialize,Add,AddAssign,Sum,Sub,Div,Mul,AsMut,AsRef,FromStr,From,Hash,PartialEq,PartialOrd,Eq,Ord,Debug,Copy,Clone, Display, Deserialize, Default, SubAssign, Neg)]
}

/// The default signed integer used. As small as possible.
pub type BaseInt = i32;
/// The default unsigned integer used. As small as possible.
pub type BaseUint = u32;
/// The default float used. As small as possible.
pub type BaseFloat = f32;

// -------------------------------

newtype! {
    #[allow(non_camel_case_types)]
/// A newType regarding milliHertz
pub struct mHz(pub BaseInt);}

newtype! {
    #[allow(non_camel_case_types)]
    /// A newType regarding milliVolts
pub struct mVolt(pub BaseInt);}

newtype! {
/// A newType regarding minutes
    pub struct Minutes(pub BaseInt);}

newtype! {
/// A newType regarding Watts
pub struct Watt(pub i64);}

newtype! {
            /// Currently one step = 15 minutes
pub struct Steps(pub BaseInt);}

impl Steps {
    /// Returns the current minutes per Step of the model.
    #[inline]
    pub const fn minutes_per_step() -> Minutes { Minutes(15) }

    /// Amount of minutes on the last day.
    #[inline]
    pub const fn time_of_day(steps: &Self) -> Minutes {
        let minutes_per_day = 1440;
        let total_minutes = steps.0 * Self::minutes_per_step().0;
        let minute_of_the_day = total_minutes % minutes_per_day;
        Minutes(minute_of_the_day)
    }

    /// Returns the percentage of the day passed.
    #[inline]
    pub fn percentage_of_day(steps: &Self) -> f32 {
        let minutes_per_day = 1440.0;
        Self::time_of_day(steps).0 as f32 / minutes_per_day
    }

    /// Amount of steps per day. Helper function.
    pub const fn steps_per_day() -> Self { Self(1440 / Self::minutes_per_step().0) }
}

impl From<Minutes> for Steps {
    fn from(i: Minutes) -> Self { Self(i.0 / Self::minutes_per_step().0) }
}

// -------------------------------

impl StructTraitBound for BaseInt {}
impl StructTraitBound for BaseUint {}
impl StructTraitBound for Steps {}
impl StructTraitBound for Minutes {}
impl StructTraitBound for mVolt {}
impl StructTraitBound for mHz {}
impl StructTraitBound for Watt {}

// -------------------------------

#[cfg(test)]
mod types_test {
    use super::*;

    const fn test_steps() -> (Steps, Steps) {
        let small_step: Steps = Steps(10);
        let big_step: Steps = Steps(350_000); // 10 years in 15 min
        (small_step, big_step)
    }

    #[test]
    fn time_of_day() {
        let (small_step, big_step) = &test_steps();
        assert_eq!(Steps::time_of_day(small_step), Minutes(10 * 15));

        let minutes_per_day = 1440;
        let remaining_big_step_steps_last_day = 350_000 * 15 % minutes_per_day;
        assert_eq!(Steps::time_of_day(big_step), Minutes(remaining_big_step_steps_last_day));
    }

    #[test]

    fn percentage_of_day() {
        let (small_step, big_step) = &test_steps();
        let minutes_per_day: f32 = 1440.0;
        assert!(((Steps::percentage_of_day(small_step) * 1000.0).round() / 1000.0 - 0.104).abs() < f32::EPSILON);
        assert!(
            (Steps::percentage_of_day(small_step) - Steps::time_of_day(small_step).0 as f32 / minutes_per_day).abs()
                < f32::EPSILON
        );
        assert!(
            (Steps::percentage_of_day(big_step) - Steps::time_of_day(big_step).0 as f32 / minutes_per_day).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn percentage_of_day_bounds() {
        let _range_test: Vec<f32> = (0..=96).map(|step| (Steps::percentage_of_day(&Steps(step)))).collect();
    }
}
