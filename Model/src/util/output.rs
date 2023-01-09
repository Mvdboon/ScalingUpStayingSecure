mod avro;
mod avrovalue;
mod graphviz;
mod json;

use std::sync::Arc;

pub use avro::*;
pub use avrovalue::*;
pub use graphviz::*;
pub use json::*;
use parking_lot::RwLock;
use rand::rngs::SmallRng;
use tokio::task::JoinHandle;

use crate::model::Model;
use crate::util::{ModelError, Steps};
use rand::SeedableRng;

impl Model {
    /// Output the model in its current state.
    pub fn output(&self, step: Steps, handles: &mut Vec<JoinHandle<()>>) -> Result<(), ModelError> {
        match self.param.type_output.as_str() {
            "json" => {
                let agent_values = Self::output_step_agents_json(&self.agents);
                Self::output_step_model_json(&self.param, step, agent_values, handles);
            }
            "avro" => {
                let agent_values = Self::output_step_agents_avro(&self.agents);
                Self::output_step_model_avro(&self.param, step, agent_values, handles);
            }
            _ => {
                return Err(ModelError::ParamError {
                    msg:     "No valid output data type".to_string(),
                    context: self.param.type_output.clone(),
                })
            }
        }
        Ok(())
    }
}

pub(crate) fn default_smallrng() -> Arc<RwLock<SmallRng>>{
    Arc::new(RwLock::new(SmallRng::seed_from_u64(117)))
}