use std::path::Path;
use std::sync::Arc;

use configparser::ini::Ini;
use parking_lot::RwLock;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::attack::{Attack, AttackBehaviour};
use crate::util::{gen_vec_attack, subparse, BaseFloat, BaseInt, ConfigError, Steps};

impl Attack {
    /// Create an [Attack] struct of the given filepath. Allows for an override for the variant if desired. It creates a
    /// new RNG seed from the RNG seed given.
    pub fn from_config(filepath: impl AsRef<Path>, variant: &str, rngseed: u64) -> Result<Self, ConfigError> {
        let mut config = Ini::new();
        match config.load(&filepath) {
            Ok(_) => (),
            Err(e) => return Err(ConfigError::LoadError(e)),
        };

        let percentage_vuln_devices: f32 = subparse::<f32>("percentage_vuln_devices", &config, variant)?;
        let infection_rate_per_step: f32 = subparse::<f32>("infection_rate_per_step", &config, variant)?;
        let patch_rate_per_step: f32 = subparse::<f32>("patch_rate_per_step", &config, variant)?;
        let infection_start: Steps = Steps(subparse::<i32>("infection_start", &config, variant)?);
        let infection_stop: Steps = Steps(subparse::<i32>("infection_stop", &config, variant)?);
        let patch_start: Steps = Steps(subparse::<i32>("patch_start", &config, variant)?);
        let patch_stop: Steps = Steps(subparse::<i32>("patch_stop", &config, variant)?);
        let attack_behaviour_out: Vec<(BaseInt, BaseInt, BaseFloat, BaseFloat)> =
            gen_vec_attack::<BaseInt, BaseFloat>(&subparse::<String>("attack_behaviour", &config, variant)?)?;

        let attack_behaviour = attack_behaviour_out
            .into_iter()
            .map(|(begin, end, report_modifier, generation_modifier)| AttackBehaviour {
                begin: Steps(begin),
                end: Steps(end),
                report_modifier,
                generation_modifier,
            })
            .collect();
        Ok(Self {
            infection_rate_per_step,
            patch_rate_per_step,
            infection_start,
            infection_stop,
            patch_start,
            patch_stop,
            attack_behaviour,
            percentage_vuln_devices,
            seed: Arc::new(RwLock::new(SmallRng::seed_from_u64(rngseed))),
            current_attack: None,
        })
    }

    /// Creates a test version to be used for testing within the crate.
    pub(crate) fn test() -> Self {
        Self {
            infection_rate_per_step: 0.002,
            patch_rate_per_step:     0.002,
            infection_start:         Steps(100),
            infection_stop:          Steps(1000),
            patch_start:             Steps(800),
            patch_stop:              Steps(60000),
            attack_behaviour:        vec![AttackBehaviour {
                begin:               Steps(10),
                end:                 Steps(100),
                report_modifier:     2.0,
                generation_modifier: 0.5,
            }],
            percentage_vuln_devices: 0.5,
            seed:                    Arc::new(RwLock::new(SmallRng::seed_from_u64(2010))),
            current_attack:          None,
        }
    }
}
