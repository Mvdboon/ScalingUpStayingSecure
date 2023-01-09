mod attackparameters;
mod gridparameters;
mod modelparameters;

use std::{fmt::{Debug, Display}};
use std::str::FromStr;

pub use attackparameters::*;
use configparser::ini::Ini;
pub use gridparameters::*;
pub use modelparameters::*;

use crate::util::ConfigError;

pub fn subparse<T>(key: &str, config: &Ini, variant: &str) -> Result<T, ConfigError>
where
    T: FromStr + Debug,
    <T as FromStr>::Err: Display,
{
    config.get(variant, key).map_or_else(
        || {
            Err(ConfigError::KeyEmptyError(format!(
                "could not get key {key} of variant {variant}"
            )))
        },
        |mut v| {
            
            v.remove_matches('_');
            match v.parse::<T>() {
                Ok(value) => Ok(value),
                Err(e) => Err(ConfigError::ParsingError(format!("{v:?} - {:?}", e.to_string()))),
            }
        },
    )
}

pub fn gen_vec_tuples_string<T>(s: &str) -> Result<Vec<(T, T)>, ConfigError>
where
    T: FromStr,
    <T as FromStr>::Err: Debug + Display,
{
    let trim: &[_] = &['[', ']', '(', ')'];
    let s1 = s.trim_matches(trim);
    let s2 = s1.replace(' ', "");
    let pairs: Vec<&str> = s2.split("),(").collect();
    let mut res = vec![];
    for pair in pairs {
        if let Some((t1, t2)) = pair.split_once(',') {
            let r1 = match t1.parse::<T>() {
                Ok(r) => r,
                Err(e) => return Err(ConfigError::ParsingError(format!("{t1:?} - {:?}", e.to_string()))),
            };

            let r2 = match t2.parse::<T>() {
                Ok(r) => r,
                Err(e) => return Err(ConfigError::ParsingError(format!("{t2:?} - {:?}", e.to_string()))),
            };
            res.push((r1, r2));
        }
    }
    Ok(res)
}

pub fn gen_vec_attack<T, L>(s: &str) -> Result<Vec<(T, T, L, L)>, ConfigError>
where
    T: FromStr,
    L: FromStr,
    <T as FromStr>::Err: Display + Debug,
    <L as FromStr>::Err: Display + Debug,
{
    let trim: &[_] = &['[', ']', '(', ')'];
    let s1 = s.trim_matches(trim);
    let s2 = s1.replace(' ', "");
    let pairs: Vec<&str> = s2.split("),(").collect();
    let mut res = vec![];
    for pair in pairs {
        let splitted: Vec<&str> = pair.split(',').collect();
        if splitted.len() < 4 {
            return Err(ConfigError::NumElementsTooLow(format!(
                "Number of elements is too low. Expected 4, got {}",
                splitted.len()
            )));
        }

        let start = match splitted[0].parse::<T>() {
            Ok(v) => v,
            Err(e) => {
                return Err(ConfigError::ParsingError(format!(
                    "{:?} - {:?}",
                    splitted[0],
                    e.to_string()
                )))
            }
        };
        let end = match splitted[1].parse::<T>() {
            Ok(v) => v,
            Err(e) => {
                return Err(ConfigError::ParsingError(format!(
                    "{:?} - {:?}",
                    splitted[1],
                    e.to_string()
                )))
            }
        };
        let gen_mod = match splitted[2].parse::<L>() {
            Ok(v) => v,
            Err(e) => {
                return Err(ConfigError::ParsingError(format!(
                    "{:?} - {:?}",
                    splitted[2],
                    e.to_string()
                )))
            }
        };
        let report_mod = match splitted[3].parse::<L>() {
            Ok(v) => v,
            Err(e) => {
                return Err(ConfigError::ParsingError(format!(
                    "{:?} - {:?}",
                    splitted[3],
                    e.to_string()
                )))
            }
        };

        res.push((start, end, gen_mod, report_mod));
    }
    Ok(res)
}
