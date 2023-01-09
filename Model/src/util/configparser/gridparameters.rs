use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use configparser::ini::Ini;

use crate::grid::{Boundaries, BoundaryBand, BoundaryUnitTrait, Grid, GridBoundaryState, NormalBand};
use crate::util::{gen_vec_tuples_string, mHz, mVolt, subparse, BaseFloat, BaseInt, ConfigError, Minutes, Steps, UtilError, Watt};

impl Grid {
    /// Creates a test version to be used for testing within the crate.
    pub fn _test() -> Self {
        Self {
            n_areas:                                  1,
            energy_storage:                           Watt(10_000),
            max_gen_inc_tick:                         Watt(100),
            ns_per_a:                                 (2, 3),
            hs_per_ns:                                (2, 3),
            pv_adoption:                              0.5,
            percentage_noise_on_power:                0.1,
            num_noise_functions:                      3,
            household_power_consumption_distribution: (Watt(10_000), Watt(100_000)),
            bulk_consumption:                         Watt(10_000),
            volt_modifier:                            1.0,
            volt_boundary:                            Boundaries::<mVolt>::default(),
            freq_boundary:                            Boundaries::<mHz>::default(),
            percentage_generation_of_usage:           0.2,
        }
    }

    /// Create a [Grid] struct of the given file path. Allows for an override for the variant if desired.
    pub fn from_config(filepath: impl AsRef<Path>, variant: &str) -> Result<Self, ConfigError> {
        let mut config = Ini::new();
        let _res = match config.load(&filepath) {
            Ok(res) => res,
            Err(e) => return Err(ConfigError::LoadError(e)),
        };
        let volt_modifier: BaseFloat = subparse("attack_modifier", &config, "voltage")?;

        let n_areas: BaseInt = subparse("n_areas", &config, variant)?;
        let energy_storage: Watt = subparse("energy_storage", &config, variant)?;
        let pv_adoption: BaseFloat = subparse::<BaseFloat>("pv_adoption", &config, variant)?;
        let max_gen_inc_tick: BaseInt = subparse("max_gen_inc_tick", &config, variant)?;
        let percentage_noise_on_power: f32 = subparse::<BaseFloat>("percentage_noise_on_power", &config, variant)?;
        let percentage_generation_of_usage: f32 =
            subparse::<BaseFloat>("percentage_generation_of_usage", &config, variant)?;
        let num_noise_functions: i32 = subparse("num_noise_functions", &config, variant)?;
        let bulk_consumption: Watt = subparse("bulk_consumption", &config, variant)?;
        let ns_per_a: (BaseInt, BaseInt) =
            gen_vec_tuples_string::<BaseInt>(&subparse::<String>("ns_per_a", &config, variant)?)?[0];
        let hs_per_ns: (BaseInt, BaseInt) =
            gen_vec_tuples_string::<BaseInt>(&subparse::<String>("hs_per_ns", &config, variant)?)?[0];
        let power_consumption_bounds: (Watt, Watt) =
            gen_vec_tuples_string::<Watt>(&subparse::<String>("power_consumption_bounds", &config, variant)?)?[0];

        let volt_boundary = Boundaries::<mVolt>::from_config_file(&filepath, "voltage")?;
        let freq_boundary = Boundaries::<mHz>::from_config_file(&filepath, "frequency")?;
        Ok(Self {
            n_areas,
            energy_storage,
            ns_per_a,
            hs_per_ns,
            pv_adoption,
            num_noise_functions,
            household_power_consumption_distribution: power_consumption_bounds,
            percentage_noise_on_power,
            max_gen_inc_tick: Watt(max_gen_inc_tick as i64),
            bulk_consumption,
            volt_modifier,
            volt_boundary,
            freq_boundary,
            percentage_generation_of_usage,
        })
    }
}

impl<T: BoundaryUnitTrait> Boundaries<T> {
    /// Creates boundary from a config file.
    pub fn from_config_file(filepath: impl AsRef<Path>, section: &str) -> Result<Self, ConfigError> {
        let mut file = BufReader::new(match File::open(filepath) {
            Ok(f) => f,
            Err(e) => return Err(UtilError::FileSystemError(e.to_string()).into()),
        });
        let mut config_string = String::new();
        match file.read_to_string(&mut config_string) {
            Ok(_) => (),
            Err(e) => return Err(UtilError::IOError { source: e }.into()),
        };
        Self::from_config_string(config_string, section)
    }

    fn from_config_string(config_string: String, section: &str) -> Result<Self, ConfigError> {
        let mut config = Ini::new();
        match config.read(config_string) {
            Ok(_) => (),
            Err(e) => return Err(UtilError::FileSystemError(e).into()),
        }
        let normal_low_float: BaseFloat = subparse("normal_low", &config, section)?;
        let normal_high_float: BaseFloat = subparse("normal_high", &config, section)?;
        let lowerbands_vec: Vec<(BaseFloat, BaseFloat)> =
            gen_vec_tuples_string::<BaseFloat>(&subparse::<String>("lowerbands", &config, section)?)?;
        let upperbands_vec: Vec<(BaseFloat, BaseFloat)> =
            gen_vec_tuples_string::<BaseFloat>(&subparse::<String>("upperbands", &config, section)?)?;

        let normal_low: T = ((normal_low_float * 1000.0) as BaseInt).into();
        let normal_high: T = ((normal_high_float * 1000.0) as BaseInt).into();

        let mut lowerbands: Vec<BoundaryBand<T>> = vec![];
        for (t, minutes) in &lowerbands_vec {
            let border: T = ((t * 1000.0) as BaseInt).into();
            let max_time_allowed: Steps = Minutes(((minutes * 1000.0) as BaseInt) / 1000).into();
            lowerbands.push(BoundaryBand {
                border,
                max_time_allowed,
                time_passed: Steps(0),
            });
        }

        let mut upperbands: Vec<BoundaryBand<T>> = vec![];
        for (t, minutes) in &upperbands_vec {
            let border: T = ((t * 1000.0) as BaseInt).into();
            let max_time_allowed: Steps = Minutes(((minutes * 1000.0) as BaseInt) / 1000).into();
            upperbands.push(BoundaryBand {
                border,
                max_time_allowed,
                time_passed: Steps(0),
            });
        }
        let normalband = NormalBand {
            lower:  normal_low,
            higher: normal_high,
        };
        Ok(Self {
            normalband,
            lowerbands,
            upperbands,
            state: GridBoundaryState::Normal,
        })
    }
}
