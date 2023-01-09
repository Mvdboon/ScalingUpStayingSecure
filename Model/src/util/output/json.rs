use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};

use apache_avro::Codec;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::trace;
#[cfg(not(feature = "single_thread"))]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tokio::task::JoinHandle;

use crate::agent::AgentList;
use crate::model::{Model, ModelParameters};
#[cfg(feature = "single_thread")]
use crate::norayon::prelude::*;
use crate::util::{Steps, UtilError};

impl Model {
    /// Transforms the vector of agents to a vector of JSON values in bytes.
    #[inline]
    pub fn output_step_agents_json(agents: &AgentList) -> Vec<Vec<u8>> {
        agents
            .par_iter()
            .map(|a| a.read_arc_recursive().get_json().unwrap())
            .collect()
    }

    /// Log the state of the model in this step.
    pub fn output_step_model_json(
        param: &ModelParameters,
        step: Steps,
        agent_values: Vec<Vec<u8>>,
        handles: &mut Vec<JoinHandle<()>>,
    ) {
        let f1 = param.outputdatafolder.clone();
        let f2 = param.outputdatafile.clone();
        let codec = param.outputcodec;

        handles.push(tokio::spawn(async move {
            let outputfile = {
                let folder: &str = &f1;
                let model: &str = &f2;
                let step = step.0 as u32;
                let folder_str = format!("{folder}/{model}");

                if create_dir_all(format!("{folder}/{model}")).is_ok() {
                    trace!("Folder was created: {folder_str}");
                } else {
                    trace!("Folder already excisted: {folder_str}");
                }

                let filepath = if codec == Codec::Null {
                    format!("{folder}/{model}/step{step}.json")
                } else {
                    format!("{folder}/{model}/step{step}.json.gz")
                };

                match File::create(&filepath) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(UtilError::FileSystemError(format!("Path: {filepath} - {e}",))),
                }
            }
            .unwrap();
            let mut bf = BufWriter::new(outputfile);
            if codec == Codec::Null {
                bf.write_all(b"[").unwrap();
                for entry in &agent_values {
                    bf.write_all(entry).unwrap();
                    bf.write_all(b",\n").unwrap();
                }
                bf.write_all(b"]").unwrap();
                bf.flush().unwrap();
            } else {
                let mut compresser = GzEncoder::new(bf, Compression::default());
                compresser.write_all(b"[").unwrap();
                for entry in &agent_values {
                    compresser.write_all(entry).unwrap();
                    compresser.write_all(b",\n").unwrap();
                }
                compresser.write_all(b"]").unwrap();
                compresser.flush().unwrap();
            }
        }));
    }
}

#[cfg(test)]
mod agent_tests {

    use super::*;
    use crate::agent::{Area, Household, Netstation, Root};
    use crate::grid::PowerGeneration;

    #[test]
    #[ignore]
    fn format_print_json() {
        let mut param = ModelParameters::test();
        let agent = Root::new(117, &param.grid);
        let agent2 = Area::new(117);
        let agent3 = Netstation::new(117, &param.grid);
        let agent4 = Household::new(117, PowerGeneration::new_no_pv(Some(117), &mut param).unwrap());
        let agent5 = Household::new(117, PowerGeneration::new_pv(Some(117), &mut param).unwrap());

        let s = serde_json::to_string(&agent).expect("msg");
        let s2 = serde_json::to_string(&agent2).expect("msg");
        let s3 = serde_json::to_string(&agent3).expect("msg");
        let s4 = serde_json::to_string(&agent4).expect("msg");
        let s5 = serde_json::to_string(&agent5).expect("msg");

        println!("{s}");
        println!("{s2}");
        println!("{s3}");
        println!("{s4}");
        println!("{s5}");
    }
}
