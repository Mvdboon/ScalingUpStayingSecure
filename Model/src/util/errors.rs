use std::io::Error as ioError;

use statrs::StatsError;
use thiserror::Error;
use tokio::task::JoinError;

/// Errors that can derive from the Util module.
#[derive(Error, Debug)]
pub enum UtilError {
    #[error(transparent)]
    StatsError {
        #[from]
        source: StatsError,
    },

    #[error(transparent)]
    IOError {
        #[from]
        source: ioError,
    },

    #[error(transparent)]
    LoggingError {
        #[from]
        source: fast_log::error::LogError,
    },

    #[error(transparent)]
    FormatError {
        #[from]
        source: std::fmt::Error,
    },

    #[error("FileSystemError: {0}")]
    FileSystemError(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("KeyError: {0}")]
    KeyParsingError(String),

    #[error("Empty Key: {0}")]
    KeyEmptyError(String),

    #[error("Parsing Error: {0}")]
    ParsingError(String),

    #[error("Number of elements too low: {0}")]
    NumElementsTooLow(String),

    #[error("Loading of configuration has failed: {0}")]
    LoadError(String),

    #[error("Config option is not permitted: {0}")]
    NotPermittedOption(String),

    #[error(transparent)]
    UtilError {
        #[from]
        source: UtilError,
    },
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("NodeIndexError: {0}")]
    NodeIndexError(#[from] std::num::ParseIntError),

    #[error("NeedToStop")]
    NeedToStop,

    #[error("Agent: {agent} could not log their state. Error: {source}")]
    LogStateErrorAvro { agent: String, source: apache_avro::Error },

    #[error("Agent: {agent} could not log their state. Error: {source}")]
    LogStateErrorJson { agent: String, source: serde_json::Error },

    #[error(transparent)]
    UtilError {
        #[from]
        source: UtilError,
    },

    #[error(transparent)]
    ConfigError {
        #[from]
        source: ConfigError,
    },

    #[error("Something went wrong with the graph: {0}")]
    GraphError(String),

    #[error("Attack had an error during Infection: {0}")]
    InfectionError(String),

    #[error("Writing output went wrong: {0}")]
    HandlerError(#[from] JoinError),

    #[error("ParamError: {context} - {msg}")]
    ParamError { msg: String, context: String },
}

impl From<fast_log::error::LogError> for ModelError {
    fn from(value: fast_log::error::LogError) -> Self { UtilError::from(value).into() }
}
