use apache_avro::Codec;
use log::LevelFilter;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use serde::{Serialize};

use crate::{attack::Attack};
use crate::grid::Grid;
use crate::util::Steps;

/// Struct that defines the model parameters
#[derive(Debug, Serialize)]
pub struct ModelParameters {
    /// Naming for the model. Used for logs etc.
    pub name:               String,
    /// Number of steps in the Model
    pub steps:              Steps,
    /// Seed for the random generators, same
    /// behaviour between runs
    #[serde(skip)]
    #[serde(default = "default_smallrng")]
    pub seed:               SmallRng,
    /// Folder to store logs
    pub logfolder:          String,
    /// Chrono - format string to determine the filename. [link](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
    pub logfile:            String,
    /// Enable the output of the node states per step.
    pub enable_output:      bool,
    /// Type of output, avro or json.
    pub type_output:        String,
    /// Folder to store the output
    pub outputdatafolder:   String,
    /// Chrono - format string to determine the filename. [link](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
    pub outputdatafile:     String,
    /// The codec that is used. Currently supported: "snappy", "deflate",
    /// 'null", See also [Codec].
    #[serde(skip)]
    pub outputcodec:        Codec,
    /// The attack struct of this model. See [Attack].
    #[serde(skip)]
    pub attack:             Attack,
    /// The Grid struct of this model. See [Grid].
    #[serde(skip)]
    pub grid:               Grid,
    /// The log level of the executable.
    #[serde(skip)]
    pub loglevel:           LevelFilter,
    /// Stop running the model when there is a grid frequency error.
    pub stop_on_freq_error: bool,
}

impl ModelParameters {
    /// Creates a test version to be used for testing within the crate.
    pub fn test() -> Self {
        Self {
            name:               "Test".to_string(),
            seed:               SmallRng::seed_from_u64(117),
            steps:              Steps(96 * 365),
            logfolder:          "logs".to_string(),
            logfile:            chrono::Local::now().format("%Y%m%d-%H_%M_%S").to_string(),
            enable_output:      false,
            outputdatafolder:   "data".to_string(),
            outputdatafile:     chrono::Local::now().format("%Y%m%d-%H_%M_%S").to_string(),
            outputcodec:        Codec::Null,
            attack:             Attack::test(),
            grid:               Grid::_test(),
            loglevel:           LevelFilter::Debug,
            type_output:        "json".to_string(),
            stop_on_freq_error: false,
        }
    }
}
