use std::collections::HashMap;

use apache_avro::schema::derive::AvroSchemaComponent;
use apache_avro::schema::{Name, Namespace};
use apache_avro::types::Value as AvroValue;
use apache_avro::Schema;

use crate::agent::AgentKind;
use crate::grid::{Boundaries, BoundaryBand, FreqState, GridBoundaryState, InfectionState, PowerGeneration, PowerState, SineParam, VoltState};
use crate::util::{mHz, mVolt, BaseInt, Steps, Watt};

type Names = HashMap<Name, Schema>;
impl AvroSchemaComponent for Steps {
    fn get_schema_in_ctxt(named_schemas: &mut Names, enclosing_namespace: &Namespace) -> Schema {
        BaseInt::get_schema_in_ctxt(named_schemas, enclosing_namespace)
    }
}
impl AvroSchemaComponent for Watt {
    fn get_schema_in_ctxt(named_schemas: &mut Names, enclosing_namespace: &Namespace) -> Schema {
        i64::get_schema_in_ctxt(named_schemas, enclosing_namespace)
    }
}
impl AvroSchemaComponent for mHz {
    fn get_schema_in_ctxt(named_schemas: &mut Names, enclosing_namespace: &Namespace) -> Schema {
        BaseInt::get_schema_in_ctxt(named_schemas, enclosing_namespace)
    }
}
impl AvroSchemaComponent for mVolt {
    fn get_schema_in_ctxt(named_schemas: &mut Names, enclosing_namespace: &Namespace) -> Schema {
        BaseInt::get_schema_in_ctxt(named_schemas, enclosing_namespace)
    }
}

impl From<&PowerState> for AvroValue {
    fn from(value: &PowerState) -> Self {
        let powerstate: Vec<(String, Self)> = vec![
            ("power_used".to_string(), Self::Long(value.power_used.0)),
            ("power_reported".to_string(), Self::Long(value.power_reported.0)),
            ("power_error".to_string(), Self::Long(value.power_error.0)),
            (
                "history_power_used".to_string(),
                Self::Array(value.history_power_used.iter().map(|a| Self::Long(a.0)).collect()),
            ),
            (
                "history_power_reported".to_string(),
                Self::Array(value.history_power_reported.iter().map(|a| Self::Long(a.0)).collect()),
            ),
            (
                "history_power_error".to_string(),
                Self::Array(value.history_power_error.iter().map(|a| Self::Long(a.0)).collect()),
            ),
            ("history_len".to_string(), Self::Long(i64::from(value.history_len))),
        ];
        Self::Record(powerstate)
    }
}
impl From<&InfectionState> for AvroValue {
    fn from(value: &InfectionState) -> Self {
        match value {
            InfectionState::NotVulnerable => Self::Enum(0, "NotVulnerable".to_string()),
            InfectionState::Vulnerable => Self::Enum(1, "Vulnerable".to_string()),
            InfectionState::Infected => Self::Enum(2, "Infected".to_string()),
            InfectionState::Patched => Self::Enum(3, "Patched".to_string()),
        }
    }
}
impl From<&PowerGeneration> for AvroValue {
    fn from(value: &PowerGeneration) -> Self {
        let cp: Vec<Self> = value.consumption_param.iter().map(Self::from).collect();
        let np: Vec<Self> = value.consumption_noise_param.iter().map(Self::from).collect();
        let res: Vec<(String, Self)> = vec![
            ("infection_state".to_string(), Self::from(&value.infection_state)),
            ("consumption_param".to_string(), Self::Array(cp)),
            ("noise_param".to_string(), Self::Array(np)),
            ("index".to_string(), (value.index).into()),
        ];
        Self::Record(res)
    }
}
impl From<&SineParam> for AvroValue {
    fn from(value: &SineParam) -> Self {
        let res: Vec<(String, Self)> = vec![
            ("a".to_string(), Self::Float(value.a)),
            ("b".to_string(), Self::Float(value.b)),
            ("c".to_string(), Self::Float(value.c)),
            ("d".to_string(), Self::Float(value.d)),
        ];
        Self::Record(res)
    }
}
impl From<&Boundaries<mVolt>> for AvroValue {
    fn from(value: &Boundaries<mVolt>) -> Self {
        let normalband = vec![
            ("lower".to_string(), Self::Int(value.normalband.lower.0)),
            ("higher".to_string(), Self::Int(value.normalband.higher.0)),
        ];
        let state = match value.state {
            GridBoundaryState::TooLow => Self::Enum(0, "TooLow".to_string()),
            GridBoundaryState::Low => Self::Enum(1, "Low".to_string()),
            GridBoundaryState::Normal => Self::Enum(2, "Normal".to_string()),
            GridBoundaryState::High => Self::Enum(3, "High".to_string()),
            GridBoundaryState::TooHigh => Self::Enum(4, "TooHigh".to_string()),
        };
        let lowerband: Vec<Self> = value.lowerbands.iter().map(Self::from).collect();
        let upperband: Vec<Self> = value.upperbands.iter().map(Self::from).collect();
        let res: Vec<(String, Self)> = vec![
            ("state".to_string(), state),
            ("normalband".to_string(), Self::Record(normalband)),
            ("lowerbands".to_string(), Self::Array(lowerband)),
            ("upperbands".to_string(), Self::Array(upperband)),
        ];
        Self::Record(res)
    }
}
impl From<&GridBoundaryState> for AvroValue {
    fn from(value: &GridBoundaryState) -> Self {
        match value {
            GridBoundaryState::TooLow => Self::Enum(0, "TooLow".to_string()),
            GridBoundaryState::Low => Self::Enum(1, "Low".to_string()),
            GridBoundaryState::Normal => Self::Enum(2, "Normal".to_string()),
            GridBoundaryState::High => Self::Enum(3, "High".to_string()),
            GridBoundaryState::TooHigh => Self::Enum(4, "TooHigh".to_string()),
        }
    }
}
impl From<&Boundaries<mHz>> for AvroValue {
    fn from(value: &Boundaries<mHz>) -> Self {
        let normalband = vec![
            ("lower".to_string(), Self::Int(value.normalband.lower.0)),
            ("higher".to_string(), Self::Int(value.normalband.higher.0)),
        ];
        let lowerband: Vec<Self> = value.lowerbands.iter().map(Self::from).collect();
        let upperband: Vec<Self> = value.upperbands.iter().map(Self::from).collect();
        let res: Vec<(String, Self)> = vec![
            ("state".to_string(), (&value.state).into()),
            ("normalband".to_string(), Self::Record(normalband)),
            ("lowerbands".to_string(), Self::Array(lowerband)),
            ("upperbands".to_string(), Self::Array(upperband)),
        ];
        Self::Record(res)
    }
}
impl From<&BoundaryBand<mVolt>> for AvroValue {
    fn from(value: &BoundaryBand<mVolt>) -> Self {
        let res: Vec<(String, Self)> = vec![
            ("border".to_string(), Self::Int(value.border.0)),
            ("max_time_allowed".to_string(), Self::Int(value.max_time_allowed.0)),
            ("time_passed".to_string(), Self::Int(value.time_passed.0)),
        ];
        Self::Record(res)
    }
}
impl From<&BoundaryBand<mHz>> for AvroValue {
    fn from(value: &BoundaryBand<mHz>) -> Self {
        let res: Vec<(String, Self)> = vec![
            ("border".to_string(), Self::Int(value.border.0)),
            ("max_time_allowed".to_string(), Self::Int(value.max_time_allowed.0)),
            ("time_passed".to_string(), Self::Int(value.time_passed.0)),
        ];
        Self::Record(res)
    }
}
impl From<&VoltState> for AvroValue {
    fn from(value: &VoltState) -> Self {
        let history: Vec<Self> = value.history.iter().map(|h| Self::Int(h.0)).collect();
        let res: Vec<(String, Self)> = vec![
            ("now".to_string(), Self::Int(value.now.0)),
            ("history".to_string(), Self::Array(history)),
            ("history_len".to_string(), Self::Int(value.history_len)),
        ];
        Self::Record(res)
    }
}
impl From<&FreqState> for AvroValue {
    fn from(value: &FreqState) -> Self {
        let history: Vec<Self> = value.history.iter().map(|h| Self::Int(h.0)).collect();
        let res: Vec<(String, Self)> = vec![
            ("now".to_string(), Self::Int(value.now.0)),
            ("history".to_string(), Self::Array(history)),
            ("history_len".to_string(), Self::Int(value.history_len)),
        ];
        Self::Record(res)
    }
}
impl From<&AgentKind> for AvroValue {
    fn from(value: &AgentKind) -> Self {
        match value {
            AgentKind::Root => Self::Enum(0, "Root".to_string()),
            AgentKind::Connection => Self::Enum(1, "Connection".to_string()),
            AgentKind::Area => Self::Enum(2, "Area".to_string()),
            AgentKind::Netstation => Self::Enum(3, "Netstation".to_string()),
            AgentKind::Household => Self::Enum(4, "Household".to_string()),
        }
    }
}
// impl From<&dyn Agent> for AvroValue {
//     fn from(value: &dyn Agent) -> Self {
//         let binding = Agent::get_schema();
//         let mut record = Record::new(&binding).unwrap();
//         record.put("kind", Self::from(&value.kind()));
//         record.put("index", Self::Long(value.index() as i64));
//         record.put("step", Self::Int(value.step().0));
//         record.put("powerstate", Self::from(&value.powerstate()));
//         let pg = value
//             .power_generation
//             .as_ref()
//             .map_or_else(|| Self::Union(0, Box::new(Self::Null)), |pg| Self::Union(1, Box::new(Self::from(pg))));
//         record.put("power_generation", pg);
//         let vb = value
//             .volt_boundary
//             .as_ref()
//             .map_or_else(|| Self::Union(0, Box::new(Self::Null)), |vb| Self::Union(1, Box::new(Self::from(vb))));
//         record.put("volt_boundary", vb);
//         let fb = value
//             .freq_boundary
//             .as_ref()
//             .map_or_else(|| Self::Union(0, Box::new(Self::Null)), |fb| Self::Union(1, Box::new(Self::from(fb))));
//         record.put("freq_boundary", fb);
//         record.put("volt_state", Self::from(&value.volt_state));
//         record.put("freq_state", Self::from(&value.freq_state));
//         record.into()
//     }
// }
