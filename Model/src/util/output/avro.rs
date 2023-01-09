use std::fs::{create_dir_all, File};
use std::io::Write;

use apache_avro::schema::UnionSchema;
use apache_avro::types::Value as AvroValue;
use apache_avro::{AvroSchema, Schema, Writer};
use log::trace;
#[cfg(not(feature = "single_thread"))]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tokio::task::JoinHandle;

use crate::agent::{AgentKind, AgentList, Area, Household, Netstation, Root};
use crate::grid::{Boundaries, PowerState};
use crate::model::{Model, ModelParameters};
#[cfg(feature = "single_thread")]
use crate::norayon::prelude::*;
use crate::util::{mHz, Steps, UtilError};

/// Get the AvroSchema needed to write the files. Is not stable yet.
pub fn get_avro_schema() -> Schema {
    let subrecords = vec![
        (" \"AgentKind\"", AgentKind::get_schema().canonical_form()),
        (" \"PowerState\"", PowerState::get_schema().canonical_form()),
        (" \"Boundaries\"", Boundaries::<mHz>::get_schema().canonical_form()),
    ];

    let root = Root::get_schema().canonical_form();
    let area = Area::get_schema().canonical_form();
    let ns = Netstation::get_schema().canonical_form();
    let hh = Household::get_schema().canonical_form();
    let mut agents = vec![root, area, ns, hh];

    // println!("{}", serde_json::to_string(&agents[0]).unwrap());

    for agent in &mut agents {
        for (name, to_replace) in &subrecords {
            *agent = agent.replace(to_replace, name);
        }
    }

    let mut str_schema: Vec<&str> = vec![];
    subrecords.iter().for_each(|(_, schema)| str_schema.push(schema));
    agents.iter().for_each(|schema| str_schema.push(schema));

    let schema_vec = Schema::parse_list(&str_schema).unwrap();

    let schema_inner = UnionSchema::new(schema_vec).unwrap();
    let s = Schema::Union(schema_inner);
    println!("{:#?}", &s);
    s
}

impl Model {
    /// Output the state of the model in avro format by transforming each agent into an AvroValue.
    pub fn output_step_agents_avro(agents: &AgentList) -> Vec<(Schema, AvroValue)> {
        agents
            .par_iter()
            .map(|a| a.read_arc_recursive().get_avro().unwrap())
            .collect()
    }

    /// Log the state of the model in this step.
    pub fn output_step_model_avro(
        param: &ModelParameters,
        step: Steps,
        agent_values: Vec<(Schema, AvroValue)>,
        handles: &mut Vec<JoinHandle<()>>,
    ) {
        let f1 = param.outputdatafolder.clone();
        let f2 = param.outputdatafile.clone();
        let codec = param.outputcodec;

        handles.push(tokio::spawn(async move {
            let mut outputfile = {
                let folder: &str = &f1;
                let model: &str = &f2;
                let step = step.0 as u32;
                let folder_str = format!("{folder}/{model}");
                if create_dir_all(format!("{folder}/{model}")).is_ok() {
                    trace!("Folder was created: {folder_str}");
                } else {
                    trace!("Folder already excisted: {folder_str}");
                }
                let filepath = format!("{folder}/{model}/step{step}.avro");
                match File::create(&filepath) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(UtilError::FileSystemError(format!("Path: {filepath} - {e}",))),
                }
            }
            .unwrap();
            let mut output_vec: Vec<u8> = vec![];
            let schema = Schema::Boolean; //// TODO: ...
            let mut writer = Writer::with_codec(&schema, &mut output_vec, codec);
            for a in agent_values {
                writer.append_value_ref(&a.1).unwrap();
            }
            writer.flush().unwrap();
            outputfile.write_all(&output_vec).unwrap();
        }));
    }
}

#[cfg(test)]
mod agent_tests {

    use apache_avro::Writer;

    use super::*;
    use crate::grid::PowerGeneration;

    // null - No value.
    // boolean - A binary value.
    // int - A 32-bit signed integer.
    // long - A 64-bit signed integer.
    // float - A single precision (32-bit) IEEE 754 floating-point number.
    // double - A double precision (64-bit) IEEE 754 floating-point number.
    // bytes - A sequence of 8-bit unsigned bytes.
    // string - A Unicode character sequence.
    #[test]
    #[ignore]
    fn format_test_avro_serialize() {
        let mut param = ModelParameters::test();
        let agent = Root::new(117, &param.grid);
        let agent2 = Area::new(117);
        let agent3 = Netstation::new(117, &param.grid);
        let agent4 = Household::new(117, PowerGeneration::new_no_pv(Some(117), &mut param).unwrap());
        let agent5 = Household::new(117, PowerGeneration::new_pv(Some(117), &mut param).unwrap());

        let mut out = vec![];
        let schema = get_avro_schema();
        println!("Pre writer");
        let mut writer = Writer::new(&schema, &mut out);
        println!("After writer");
        writer.append_ser(agent).expect("error agent no gen");
        writer.append_ser(agent3).expect("error agent pv gen");
        writer.append_ser(agent2).expect("error agent no pv gen");
        writer.append_ser(agent4).expect("error agent no pv gen");
        writer.append_ser(agent5).expect("error agent no pv gen");
        writer.flush().expect(":");
    }

    #[test]
    #[ignore]
    fn output_avro_schema() {
        let mut file = File::create("avro_schema.json").unwrap();
        // let output = format!("{:?}", get_avro_schema());
        let output = serde_json::to_string(&get_avro_schema()).unwrap();
        file.write_all(output.as_bytes()).unwrap();
    }

    #[test]
    #[ignore]
    fn output_avro_clean_testmodel() {
        let mut param = ModelParameters::test();
        param.enable_output = true;
        param.type_output = "avro".to_string();
        param.outputdatafile = "testmodel".to_string();
        let mut model = Model::new(param).unwrap();
        model.step(Steps(10)).expect("Error in taking steps");
    }
}
