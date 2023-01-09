use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
/// The state the grid is in.
#[derive(PartialOrd, Deserialize, Ord, PartialEq, Eq, Clone, Copy, Debug, Serialize, AvroSchema, Default)]
pub enum GridBoundaryState {
    TooLow,
    Low,
    #[default]
    Normal,
    High,
    TooHigh,
}
