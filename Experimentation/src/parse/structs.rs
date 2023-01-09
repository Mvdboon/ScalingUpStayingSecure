use itertools::Itertools;
use serde::Serialize;
use serde_json::Value;
use smartgrid_iot_security::attack::Attack;
use smartgrid_iot_security::grid::Grid;

use crate::creator::Context;
use crate::parse::functions::extract_time_between;

#[derive(Debug, Serialize, Default)]
pub struct FileAnswer {
    pub first_freq_warn: Option<(i32, Value)>,
    pub first_freq_err:  Option<(i32, Value)>,
    pub first_volt_warn: Option<(i32, Vec<Value>)>,
    pub first_volt_err:  Option<(i32, Vec<Value>)>,
    pub warn_err:        Vec<GridWarning>,
    pub reserve_power:   Vec<ReservePower>,
    pub frequency:       Vec<GridFrequency>,
}

#[derive(Debug, Serialize, Default)]
pub struct ExperimentResult {
    // Parameters
    name:                           String,
    seed:                           u64,
    percentage_vuln_devices:        f32,
    percentage_generation_of_usage: f32,
    pv_adoption:                    f32,
    max_gen_inc_tick:               i64,
    energy_storage:                 i64,
    bulk_consumption:               i64,
    // Time results
    time_to_warn_freq:              Option<i32>,
    time_to_err_freq:               Option<i32>,
    time_to_warn_volt:              Option<i32>,
    time_to_err_volt:               Option<i32>,
    time_between_freq:              Option<i32>,
    time_between_volt:              Option<i32>,
    // Extreme values
    most_extreme_low_volt:          Option<i64>,
    most_extreme_high_volt:         Option<i64>,
    most_extreme_low_freq:          Option<i64>,
    most_extreme_high_freq:         Option<i64>,
    most_low_reserve_power:         Option<i64>,
    most_high_reserve_power:        Option<i64>,
    
    // Amount of err/warn
    amount_warn_freq:               i64,
    amount_err_freq:                i64,
    amount_warn_volt:               i64,
    amount_err_volt:                i64,
    // Count
    unique_agents_volt_warn:        i64,
    unique_agents_volt_err:         i64,
}

impl ExperimentResult {
    pub fn create(context: Context, file_answer: &FileAnswer) -> Self {
        let TimeBetween {
            time_between_volt,
            time_between_freq,
        } = extract_time_between(file_answer);

        let mut res = Self {
            name: context.name,
            seed: context.seed,
            percentage_vuln_devices: context.percentage_vuln_devices,
            percentage_generation_of_usage: context.percentage_generation_of_usage,
            pv_adoption: context.pv_adoption,
            max_gen_inc_tick: context.max_gen_inc_tick,
            energy_storage: context.energy_storage,
            bulk_consumption: context.bulk_consumption,
            time_to_warn_freq: file_answer.first_freq_err.as_ref().map(|v| v.0),
            time_to_err_freq: file_answer.first_freq_err.as_ref().map(|v| v.0),
            time_to_warn_volt: file_answer.first_volt_warn.as_ref().map(|v| v.0),
            time_to_err_volt: file_answer.first_volt_err.as_ref().map(|v| v.0),
            time_between_freq,
            time_between_volt,
            ..Default::default()
        };
        res.unique_agents_volt_warn = file_answer
            .warn_err
            .iter()
            .filter_map(|we| {
                if !we.critical && we.kind.as_str() == "volt" {
                    Some(we.agent_index.to_string())
                } else {
                    None
                }
            })
            .unique()
            .count() as i64;
        res.unique_agents_volt_err = file_answer
            .warn_err
            .iter()
            .filter_map(|we| {
                if we.critical && we.kind.as_str() == "volt" {
                    Some(we.agent_index.to_string())
                } else {
                    None
                }
            })
            .unique()
            .count() as i64;

        for item in &file_answer.warn_err {
            match (item.kind.as_str(), item.critical) {
                ("freq", true) => res.amount_err_freq += 1,
                ("freq", false) => res.amount_warn_freq += 1,
                ("volt", true) => res.amount_err_volt += 1,
                ("volt", false) => res.amount_warn_volt += 1,
                _ => panic!("Could not identify: {item:?}"),
            };
            let current_value: i64 = item.current_value.to_string().parse().unwrap();

            match item.kind.as_str() {
                "freq" => {},
                //     if res.most_extreme_high_freq.is_none() {
                //         res.most_extreme_high_freq = Some(current_value);
                //     }
                //     if let Some(v) = &mut res.most_extreme_high_freq && current_value>*v{
                //     *v=current_value;
                // }
                //     if res.most_extreme_low_freq.is_none() {
                //         res.most_extreme_low_freq = Some(current_value);
                //     }
                //     if let Some(v) = &mut res.most_extreme_low_freq && current_value<*v{
                //     *v=current_value;
                // }
                // }
                "volt" => {
                    if res.most_extreme_high_volt.is_none() {
                        res.most_extreme_high_volt = Some(current_value);
                    }
                    if let Some(v) = &mut res.most_extreme_high_volt && current_value>*v{
                    *v=current_value;
                }
                    if res.most_extreme_low_volt.is_none() {
                        res.most_extreme_low_volt = Some(current_value);
                    }
                    if let Some(v) = &mut res.most_extreme_low_volt && current_value<*v{
                    *v=current_value;
                }
                }
                _ => panic!("Could not identify: {item:?}"),
            };
        }

        for reserve in &file_answer.reserve_power {
            if res.most_low_reserve_power.is_none() {
                res.most_low_reserve_power = Some(reserve.current_usage);
            }
            if res.most_high_reserve_power.is_none() {
                res.most_high_reserve_power = Some(reserve.current_usage);
            }

            if let Some(v) = &mut res.most_low_reserve_power && reserve.current_usage < *v {
                *v = reserve.current_usage;
            }

            if let Some(v) = &mut res.most_high_reserve_power && reserve.current_usage > *v {
                *v = reserve.current_usage;
            }
        }

        for freq in &file_answer.frequency {
            if res.most_extreme_high_freq.is_none() {
                res.most_extreme_high_freq = Some(freq.hz);
            }
            if res.most_extreme_low_freq.is_none() {
                res.most_extreme_low_freq = Some(freq.hz);
            }

            if let Some(v) = &mut res.most_extreme_high_freq && freq.hz > *v {
                *v = freq.hz;
            }

            if let Some(v) = &mut res.most_extreme_low_freq && freq.hz < *v {
                *v = freq.hz;
            }
        }
        res
    }
}

#[derive(Serialize, Debug)]
pub struct TimeBetween {
    pub time_between_volt: Option<i32>,
    pub time_between_freq: Option<i32>,
}

#[derive(Serialize, Debug)]
pub struct GridFrequency {
    pub step: i32,
    pub hz:   i64,
}
impl GridFrequency {
    pub fn from_value(step: i32, value: &Value) -> Self {
        Self {
            step,
            hz: value["freq_state"]["now"].clone().as_i64().unwrap(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ReservePower {
    pub step:          i32,
    pub lower_limit:   i64,
    pub upper_limit:   i64,
    pub current_usage: i64,
    pub watt_per_step: i64,
}

impl ReservePower {
    pub fn from_value(step: i32, value: &Value) -> Self {
        Self {
            step,
            lower_limit: value["reserve_power"]["lower_limit"].clone().as_i64().unwrap(),
            upper_limit: value["reserve_power"]["upper_limit"].clone().as_i64().unwrap(),
            current_usage: value["reserve_power"]["current_usage"].clone().as_i64().unwrap(),
            watt_per_step: value["reserve_power"]["watt_per_step"].clone().as_i64().unwrap(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct GridWarning {
    pub step:                    i32,
    pub kind:                    String,
    pub critical:                bool,
    pub state:                   Value,
    pub agent_index:             Value,
    pub inf_num_not_vulnerable:  Value,
    pub inf_num_vulnerable:      Value,
    pub inf_num_infected:        Value,
    pub inf_num_patched:         Value,
    pub inf_perc_not_vulnerable: Value,
    pub inf_perc_vulnerable:     Value,
    pub inf_perc_infected:       Value,
    pub inf_perc_patched:        Value,
    pub power_generated:         Value,
    pub power_used:              Value,
    pub power_reported:          Value,
    pub power_error:             Value,
    pub current_value:           Value,
    pub value_history0:          Value,
    pub value_history1:          Value,
    pub value_history2:          Value,
    pub value_history3:          Value,
    pub value_history4:          Value,
    pub value_history5:          Value,
    pub value_history6:          Value,
    pub value_history7:          Value,
    pub value_history8:          Value,
    pub value_history9:          Value,
}

impl GridWarning {
    pub fn from_value(step: i32, value: &Value) -> Self {
        let (state, kind) = match (value["freq_state"].is_null(), value["volt_state"].is_null()) {
            (false, true) => (value["freq_state"].clone(), "freq".to_owned()),
            (true, false) => (value["volt_state"].clone(), "volt".to_owned()),
            _ => panic!("Could not determine kind of {value}"),
        };
        Self {
            step,
            kind,
            critical: serde_json::from_value(value["critical"].clone()).unwrap(),
            state: serde_json::from_value(value["state"].clone()).unwrap(),
            agent_index: value["agent_index"].clone(),
            power_generated: value["agent_power_state"]["power_generated"].clone(),
            power_used: value["agent_power_state"]["power_used"].clone(),
            power_reported: value["agent_power_state"]["power_reported"].clone(),
            power_error: value["agent_power_state"]["power_error"].clone(),
            current_value: state["now"].clone(),
            inf_num_not_vulnerable: value["infectionstatistics"]["num_not_vulnerable"].clone(),
            inf_num_vulnerable: value["infectionstatistics"]["num_vulnerable"].clone(),
            inf_num_infected: value["infectionstatistics"]["num_infected"].clone(),
            inf_num_patched: value["infectionstatistics"]["num_patched"].clone(),
            inf_perc_not_vulnerable: value["infectionstatistics"]["perc_not_vulnerable"].clone(),
            inf_perc_vulnerable: value["infectionstatistics"]["perc_vulnerable"].clone(),
            inf_perc_infected: value["infectionstatistics"]["perc_infected"].clone(),
            inf_perc_patched: value["infectionstatistics"]["perc_patched"].clone(),
            value_history0: state["history"].get(0).unwrap().clone(),
            value_history1: state["history"].get(1).unwrap().clone(),
            value_history2: state["history"].get(2).unwrap().clone(),
            value_history3: state["history"].get(3).unwrap().clone(),
            value_history4: state["history"].get(4).unwrap().clone(),
            value_history5: state["history"].get(5).unwrap().clone(),
            value_history6: state["history"].get(6).unwrap().clone(),
            value_history7: state["history"].get(7).unwrap().clone(),
            value_history8: state["history"].get(8).unwrap().clone(),
            value_history9: state["history"].get(9).unwrap().clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Line {
    ModelParameters { content: Value },
    AttackParameters { content: Attack },
    GridParameters { content: Grid },
    GridInformation { step: i32, content: Value },
    FrequencyWarning { step: i32, content: Value },
    FrequencyError { step: i32, content: Value },
    VoltageWarning { step: i32, content: Vec<Value> },
    VoltageError { step: i32, content: Vec<Value> },
    Step { content: i32 },
    Other,
}

impl Line {
    pub fn convert(step: i32, s: &str) -> Self {
        let mp = "INFO  - Model Param: ";
        let ap = "INFO  - Attack Param: ";
        let gp = "INFO  - Grid Param: ";
        let gi = "INFO  - Grid information - ";
        let fe = "ERROR - Frequency error - ";
        let ve = "ERROR - Voltage error - ";
        let vw = "WARN  - Voltage warning - ";
        let fw = "WARN  - Frequency warning - ";
        let st = "Taking step";
        match s {
            l if s.contains(mp) => Self::ModelParameters {
                content: serde_json::from_str(l.split_once(mp).unwrap().1).unwrap(),
            },
            l if s.contains(ap) => Self::AttackParameters {
                content: serde_json::from_str(l.split_once(ap).unwrap().1).unwrap(),
            },
            l if s.contains(gp) => Self::GridParameters {
                content: serde_json::from_str(l.split_once(gp).unwrap().1).unwrap(),
            },
            l if s.contains(gi) => Self::GridInformation {
                step,
                content: serde_json::from_str(l.split_once(gi).unwrap().1).unwrap(),
            },
            l if s.contains(st) => Self::Step {
                content: l.split(' ').last().unwrap().parse::<i32>().unwrap(),
            },
            l if s.contains(fw) => Self::FrequencyWarning {
                step,
                content: serde_json::from_str(l.split_once(fw).unwrap().1).unwrap(),
            },
            l if s.contains(fe) => Self::FrequencyError {
                step,
                content: serde_json::from_str(l.split_once(fe).unwrap().1).unwrap(),
            },
            l if s.contains(vw) => Self::VoltageWarning {
                step,
                content: serde_json::from_str(l.split_once(vw).unwrap().1).unwrap(),
            },
            l if s.contains(ve) => Self::VoltageError {
                step,
                content: serde_json::from_str(l.split_once(ve).unwrap().1).unwrap(),
            },
            _ => Self::Other,
        }
    }
}
