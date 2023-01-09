use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};

use crate::grid::{Grid, GridState};
use crate::util::{mVolt, BaseFloat, BaseInt, Watt};

/// Voltage state of an agent. With voltage, these are the Netstation agents as they transform the voltage to the
/// familiar 230V.
#[derive(Serialize, Clone, AvroSchema, Deserialize, Debug, Default)]
pub struct VoltState {
    /// Current value
    pub now:         mVolt,
    /// History of the values.
    pub history:     Vec<mVolt>,
    /// How much history is kept?
    pub history_len: BaseInt,
    volt_modifier:   BaseFloat,
}

impl GridState<mVolt> for VoltState {
    fn power_mismatch(&mut self, power_total: &Watt, power_error: &Watt, _bulk_consumption: &Watt) -> mVolt {
        mVolt(
            self.now.0
                + ((f64::from(self.now.0) * f64::from(self.volt_modifier)) * power_error.0 as f64
                    / (power_total.0) as f64) as BaseInt,
        )
    }

    fn new(grid_param: &Grid) -> Self {
        Self {
            now:           mVolt(230_000),
            history:       vec![],
            history_len:   10,
            volt_modifier: grid_param.volt_modifier,
        }
    }

    fn update(&mut self, new: mVolt) {
        self.history.push(self.now);
        self.now = new;

        if self.history.len() > self.history_len as usize {
            self.history.remove(0);
        }
    }
}
