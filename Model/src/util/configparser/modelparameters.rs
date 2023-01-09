use std::path::Path;

use apache_avro::Codec;
use configparser::ini::Ini;
use log::LevelFilter;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::attack::Attack;
use crate::grid::Grid;
use crate::model::ModelParameters;
use crate::util::{subparse, ConfigError, Steps};

impl ModelParameters {
    /// Create the `ModelParameters` from a "ModelParameters.ini"
    /// file in the directory.
    pub fn from_config(filepath: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut config = Ini::new();

        match config.load(&filepath) {
            Ok(res) => res,
            Err(e) => return Err(ConfigError::LoadError(e)),
        };

        let variant = "general";

        let name = subparse("name", &config, variant)?;
        let steps: Steps = subparse("steps", &config, variant)?;
        let seed_prime: u64 = subparse("seed", &config, variant)?;
        let logfolder: String = subparse("logfolder", &config, variant)?;
        let logfile: String = subparse("logfile", &config, variant)?;
        let enable_output: bool = subparse("enable_output", &config, variant)?;
        let stop_on_freq_error: bool = subparse("stop_on_freq_error", &config, variant)?;
        let outputdatafolder: String = subparse("outputdatafolder", &config, variant)?;
        let outputdatafile: String = subparse("outputdatafile", &config, variant)?;
        let attack_file: String = subparse("attack_file", &config, variant)?;
        let grid_file: String = subparse("grid_file", &config, variant)?;
        let grid_variant: String = subparse("grid_variant", &config, variant)?;
        let attack_variant: String = subparse("attack_variant", &config, variant)?;
        let type_output: String = subparse("type_output", &config, variant)?;
        let outputcodec: String = subparse("outputcodec", &config, variant)?;
        let outputcodec = match outputcodec.as_str() {
            "null" => Codec::Null,
            "snappy" => Codec::Snappy,
            "deflate" => Codec::Deflate,
            _ => {
                return Err(ConfigError::NotPermittedOption(format!(
                    "{outputcodec} is not a permitted codec"
                )))
            }
        };

        let loglevel: String = subparse("loglevel", &config, variant)?;
        let loglevel = match loglevel.as_str() {
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "trace" => LevelFilter::Trace,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => {
                return Err(ConfigError::NotPermittedOption(format!(
                    "{loglevel} is not a permitted LogLevel"
                )))
            }
        };

        let attack = Attack::from_config(attack_file, &attack_variant, seed_prime)?;
        let grid = Grid::from_config(grid_file, &grid_variant)?;

        Ok(Self {
            name,
            steps,
            seed: SmallRng::seed_from_u64(seed_prime),
            logfolder,
            logfile: chrono::Local::now().format(&logfile).to_string(),
            enable_output,
            outputdatafolder,
            outputdatafile: chrono::Local::now().format(&outputdatafile).to_string(),
            outputcodec,
            attack,
            grid,
            loglevel,
            type_output,
            stop_on_freq_error,
        })
    }
}
