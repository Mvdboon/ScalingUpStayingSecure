use std::fmt::Display;

use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

use crate::grid::{Grid, GridState};
use crate::util::{mHz, BaseInt, Watt};

/// Frequency state of an agent. With frequency, this is only the root node.
#[derive(Serialize, Clone, Debug, AvroSchema, Deserialize, Default)]
pub struct FreqState {
    /// Current value
    pub now:         mHz,
    /// History of the values.
    pub history:     Vec<mHz>,
    /// How much history is kept?
    pub history_len: BaseInt,
}

impl GridState<mHz> for FreqState {
    fn power_mismatch(&mut self, power_total: &Watt, power_error: &Watt, bulk_consumption: &Watt) -> mHz {
        mHz(self.now.0
            + (f64::from(self.now.0) * power_error.0 as f64 / (power_total.0 + bulk_consumption.0) as f64) as BaseInt)
    }

    fn new(_: &Grid) -> Self {
        Self {
            now:         mHz(50_000),
            history:     vec![],
            history_len: 10,
        }
    }

    fn update(&mut self, new: mHz) {
        self.history.push(self.now);
        self.now = new;

        if self.history.len() > self.history_len as usize {
            self.history.remove(0);
        }
    }
}

impl Display for FreqState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Grid Frequency")
            .field("Currently", &self.now.to_string())
            .finish()
    }
}

#[cfg(test)]
mod test_freq_state {
    use super::*;

    fn test_create_freq_state() -> FreqState {
        FreqState {
            now: mHz(50_000),
            ..Default::default()
        }
    }

    #[test]
    fn test_power_mismatch_no_bulk() {
        let mut fs = test_create_freq_state();
        let new = fs.power_mismatch(&100.into(), &10.into(), &0.into());
        assert_eq!(new, mHz(55000));

        let mut fs = test_create_freq_state();
        let new = fs.power_mismatch(&100.into(), &Watt(-10), &0.into());
        assert_eq!(new, mHz(45000));
    }

    #[test]
    fn test_power_mismatch_bulk() {
        let mut fs = test_create_freq_state();
        let new = fs.power_mismatch(&100.into(), &10.into(), &1000.into());
        assert_eq!(new, mHz(50454));

        let mut fs = test_create_freq_state();
        let new = fs.power_mismatch(&100.into(), &Watt(-10), &1000.into());
        assert_eq!(new, mHz(49546));
    }
}
