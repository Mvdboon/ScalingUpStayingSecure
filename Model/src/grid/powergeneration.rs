use std::collections::HashMap;
use std::f32::consts::PI;

use apache_avro::AvroSchema;
use derive_more::{Mul, MulAssign};
use serde::{Deserialize, Serialize};

use crate::grid::InfectionState;
use crate::model::ModelParameters;
use crate::util::{norm_dist, random_percentage, BaseFloat, BaseInt, Steps, UtilError, Watt};

/// Contains the power generation information for a household.
#[derive(Clone, Serialize, AvroSchema, Deserialize, Debug, PartialEq)]
pub struct PowerGeneration {
    /// Current state of infection. Also, the indication of the system if it is
    /// vulnerable or not.
    pub infection_state:                InfectionState,
    /// the generation parameters that create the power generated per step.
    pub generation_param:               Vec<SineParam>,
    /// The usage parameters that create the power consumed per step.
    pub consumption_param:              Vec<SineParam>,
    /// The noise parameters given to this [PowerGeneration] unit.
    pub generation_noise_param:         Vec<SineParam>,
    /// The noise parameters given to this [PowerGeneration] unit.
    pub consumption_noise_param:        Vec<SineParam>,
    /// The index of the household associated with the power generation unit.
    pub index:                          Option<BaseInt>,
    /// Average power consumption
    pub average_power_usage:            Watt,
    /// Amount of noise in relation to the average power consumption of the household.
    pub noise_percentage:               f32,
    /// The percentage of power with regards to the average consumption of the household that is being generated on average.
    pub percentage_generation_of_usage: f32,
    #[serde(skip)]
    #[avro(skip)]
    calc_power_cache:                   HashMap<Steps, (Watt, Watt, Watt)>,
}

impl PowerGeneration {
    /// Create a new PV power generation unit. Assumes that a house on average
    /// is self-sufficient.
    pub fn new_pv(index: Option<BaseInt>, param: &mut ModelParameters) -> Result<Self, UtilError> {
        let mut pg = Self::new(index, param, true)?;
        pg.generation_param = gen_generation_param(param)?;
        Ok(pg)
    }

    /// Creates a power generation unit for a house that only consumes energy.
    pub fn new_no_pv(index: Option<BaseInt>, param: &mut ModelParameters) -> Result<Self, UtilError> {
        let pg = Self::new(index, param, false)?;
        Ok(pg)
    }

    fn new(index: Option<BaseInt>, param: &mut ModelParameters, infectable: bool) -> Result<Self, UtilError> {
        let average_power_usage = Watt(
            norm_dist(
                &(param.grid.household_power_consumption_distribution.0 .0 as f32),
                &(param.grid.household_power_consumption_distribution.1 .0 as f32),
                &mut param.seed,
            )
            .expect("Couldnt get average power") as i64,
        );
        let generation_noise_param = gen_noise_param(param)?;
        let consumption_noise_param = gen_noise_param(param)?;
        let consumption_param = gen_consumption_param(param)?;
        let infection_state = if infectable && random_percentage(&mut param.seed) < param.attack.percentage_vuln_devices
        {
            InfectionState::Vulnerable
        } else {
            InfectionState::NotVulnerable
        };

        Ok(Self {
            infection_state,
            index,
            average_power_usage,
            generation_param: vec![],
            consumption_param,
            generation_noise_param,
            consumption_noise_param,
            noise_percentage: param.grid.percentage_noise_on_power,
            percentage_generation_of_usage: param.grid.percentage_generation_of_usage,
            calc_power_cache: HashMap::new(),
        })
    }

    /// Calculates the power of the unit. Both the noise and power consumption
    /// parts.
    #[inline]
    pub fn calc_power(&mut self, step: &Steps) -> (Watt, Watt, Watt) {
        match self.calc_power_cache.contains_key(step) {
            true => self.calc_power_cache.get(step).unwrap().to_owned(),
            false => {
                let new_entry = self.gen_calc_power(step);
                self.calc_power_cache.insert(step.to_owned(), new_entry);
                new_entry
            }
        }
    }

    fn gen_calc_power(&self, step: &Steps) -> (Watt, Watt, Watt) {
        let power_generated = if self.generation_param.is_empty() {
            Watt(0)
        } else {
            Watt(
                (self
                    .generation_noise(step)
                    .mul_add(self.noise_percentage, self.generation(step))
                    * self.average_power_usage.0 as f32) as i64,
            )
        };
        let power_used = Watt(
            (self
                .consumption_noise(step)
                .mul_add(self.noise_percentage, self.consumption(step))
                * self.average_power_usage.0 as f32) as i64,
        );
        (power_generated, power_used, power_used - power_generated)
    }

    #[inline]
    fn consumption(&self, step: &Steps) -> BaseFloat { Self::calc_sin(step, &self.consumption_param) }

    #[inline]
    fn generation(&self, step: &Steps) -> BaseFloat {
        Self::calc_sin(step, &self.generation_param) * self.percentage_generation_of_usage
    }

    #[inline]
    fn consumption_noise(&self, step: &Steps) -> BaseFloat { Self::calc_sin(step, &self.consumption_noise_param) }

    #[inline]
    fn generation_noise(&self, step: &Steps) -> BaseFloat { Self::calc_sin(step, &self.generation_noise_param) }

    #[inline]
    fn calc_sin(step: &Steps, sinparams: &[SineParam]) -> f32 {
        sinparams
            .iter()
            .map(|s| {
                let perc = Steps::percentage_of_day(step);
                if perc>=s.begin && perc<=s.end{
                    let b = s.b * 2.0 * PI * perc;
                    let bc =  (b + s.c).sin();
                    let ans = s.a.mul_add(bc, s.d);
                    s.minimum.map_or(ans, |v| v.max(ans))
                }else{
                    0.0
                }

            })
            // .map(|_| 1.0 * (2.0 * PI * step.0 as f32 / 95.0).sin())
            .sum()
    }
}

fn gen_consumption_param(param: &mut ModelParameters) -> Result<Vec<SineParam>, UtilError> {
    let consumption_param: Vec<SineParam> = vec![
        SineParam {
            a:       norm_dist(&0.4, &0.05, &mut param.seed)?,
            b:       1.0,
            c:       ((0.7 * 2.0) as BaseFloat).mul_add(PI, norm_dist(&0.0, &0.3, &mut param.seed)?),
            d:       0.6,
            begin:   0.0,
            end:     1.0,
            minimum: None,
        },
        SineParam {
            a:       1.0,
            b:       1.0,
            c:       0.75 * 2.0 * PI,
            d:       0.0,
            begin:   0.25,
            end:     0.75,
            minimum: None,
        },
    ];

    Ok(consumption_param)
}

fn gen_generation_param(param: &mut ModelParameters) -> Result<Vec<SineParam>, UtilError> {
    let gen_param: Vec<SineParam> = vec![SineParam {
        a:       norm_dist(&1.3, &0.3, &mut param.seed)?,
        b:       0.9,
        c:       0.775 * 2.0 * PI,
        d:       -0.3,
        begin:   0.25,
        end:     0.85,
        minimum: Some(0.0),
    }];
    Ok(gen_param)
}

fn gen_noise_param(param: &mut ModelParameters) -> Result<Vec<SineParam>, UtilError> {
    let mut noise_param: Vec<SineParam> = vec![];

    for _ in 0..param.grid.num_noise_functions {
        noise_param.push(SineParam {
            a:       norm_dist(&0.5, &0.02, &mut param.seed)?,
            b:       norm_dist(&1.0, &0.02, &mut param.seed)?,
            c:       norm_dist(&1.0, &0.05, &mut param.seed)?,
            d:       1.0,
            begin:   0.0,
            end:     1.0,
            minimum: None,
        });
    }
    Ok(noise_param)
}

/// Contains the parameters of the sine function that create the behaviour.
/// Note, the b attribute goes from 0 to 1 for a whole period.
/// y = a * sin(bx + c) + d
#[derive(Serialize, Clone, Copy, AvroSchema, Deserialize, Debug, PartialEq, Mul, MulAssign)]
pub struct SineParam {
    /// when to start
    pub begin:   f32,
    /// when to end
    pub end:     f32,
    /// Amplitude
    pub a:       f32,
    /// Period. 0 -> 1. Not 0 to 2PI.
    pub b:       f32,
    /// Horizontal shift. Position of the peak.
    pub c:       f32,
    /// Vertical shift. Average consumption throughout the day.
    pub d:       f32,
    /// minimum value allowed
    pub minimum: Option<f32>,
}

#[cfg(test)]
mod power_generation_test {

    use std::fs::File;
    use std::io::Write;

    use super::*;

    #[test]
    #[ignore]
    fn output_for_validation() {
        let mut param = ModelParameters::test();

        let powergens_pv: Vec<PowerGeneration> = (0..100)
            .map(|_| PowerGeneration::new_pv(None, &mut param).unwrap())
            .collect();

        let powergens_non_pv: Vec<PowerGeneration> = (0..100)
            .map(|_| PowerGeneration::new_no_pv(None, &mut param).unwrap())
            .collect();

        let filename = "../data_analysis/model_output/No_PV_validation_consumption.csv";
        let data: Vec<Vec<f64>> = powergens_non_pv
            .clone()
            .into_iter()
            .map(|pg| {
                let res: Vec<f64> = (0..96)
                    .into_iter()
                    .map(|step| pg.consumption(&Steps(step)) as f64)
                    .collect();
                let total: f64 = res.iter().sum();
                res.into_iter().map(|v| v / total).collect()
            })
            .collect();
        output_to_file(filename, data);

        let filename = "../data_analysis/model_output/PV_validation_consumption.csv";
        let data: Vec<Vec<f64>> = powergens_pv
            .clone()
            .into_iter()
            .map(|pg| {
                let res: Vec<f64> = (0..96)
                    .into_iter()
                    .map(|step| pg.consumption(&Steps(step)) as f64)
                    .collect();
                let total: f64 = res.iter().sum();
                res.into_iter().map(|v| v / total).collect()
            })
            .collect();
        output_to_file(filename, data);

        let filename = "../data_analysis/model_output/No_PV_validation_generation.csv";
        let data: Vec<Vec<f64>> = powergens_non_pv
            .into_iter()
            .map(|pg| {
                let res: Vec<f64> = (0..96)
                    .into_iter()
                    .map(|step| pg.generation(&Steps(step)) as f64)
                    .collect();
                let total: f64 = res.iter().sum();
                res.into_iter().map(|v| v / total).collect()
            })
            .collect();
        output_to_file(filename, data);

        let filename = "../data_analysis/model_output/PV_validation_generation.csv";
        let data: Vec<Vec<f64>> = powergens_pv
            .into_iter()
            .map(|pg| {
                let res: Vec<f64> = (0..96)
                    .into_iter()
                    .map(|step| pg.generation(&Steps(step)) as f64)
                    .collect();
                let total: f64 = res.iter().sum();
                res.into_iter().map(|v| v / total).collect()
            })
            .collect();
        output_to_file(filename, data);
    }

    fn output_to_file(filename: &'static str, data: Vec<Vec<f64>>) {
        let mut file = File::create(filename).expect("Could not open file");
        // let transform = get_step_to_time();

        // file.write_all(b"Tijd;");
        for i in 0..data.len() {
            file.write_all(i.to_string().as_bytes()).unwrap();
            file.write_all(b";").unwrap();
        }
        file.write_all(b"\n").unwrap();

        for step in 0..96 {
            // file.write_all(transform[step].as_bytes());
            // file.write_all(b";");
            for d in &data {
                file.write_all(d[step].to_string().as_bytes()).unwrap();
                file.write_all(b";").unwrap();
            }
            file.write_all(b"\n").unwrap();
        }
        file.flush().unwrap();
    }

    #[test]
    fn sine_func_test() {
        let sp = vec![SineParam {
            a:       1.0,
            b:       2.0 * PI,
            c:       0.0,
            d:       0.0,
            begin:   0.0,
            end:     1.0,
            minimum: None,
        }];
        let _: Vec<f32> = (0..=96).map(|x| PowerGeneration::calc_sin(&Steps(x), &sp)).collect();
        // println!("{ans:#?}");
    }

    fn _get_step_to_time() -> Vec<&'static str> {
        vec![
            "00:15", "00:30", "00:45", "01:00", "01:15", "01:30", "01:45", "02:00", "02:15", "02:30", "02:45", "03:00",
            "03:15", "03:30", "03:45", "04:00", "04:15", "04:30", "04:45", "05:00", "05:15", "05:30", "05:45", "06:00",
            "06:15", "06:30", "06:45", "07:00", "07:15", "07:30", "07:45", "08:00", "08:15", "08:30", "08:45", "09:00",
            "09:15", "09:30", "09:45", "10:00", "10:15", "10:30", "10:45", "11:00", "11:15", "11:30", "11:45", "12:00",
            "12:15", "12:30", "12:45", "13:00", "13:15", "13:30", "13:45", "14:00", "14:15", "14:30", "14:45", "15:00",
            "15:15", "15:30", "15:45", "16:00", "16:15", "16:30", "16:45", "17:00", "17:15", "17:30", "17:45", "18:00",
            "18:15", "18:30", "18:45", "19:00", "19:15", "19:30", "19:45", "20:00", "20:15", "20:30", "20:45", "21:00",
            "21:15", "21:30", "21:45", "22:00", "22:15", "22:30", "22:45", "23:00", "23:15", "23:30", "23:45", "00:00",
        ]
    }
}
