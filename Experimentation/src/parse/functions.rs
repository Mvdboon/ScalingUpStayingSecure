use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use csv::Writer;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;

use crate::creator::Context;
use crate::parse::structs::{ReservePower, *};

pub fn output_csv_gz<Row: Serialize>(filename: &PathBuf, rows: &[Row]) {
    let file = File::create(filename).unwrap_or_else(|_| panic!("Could not create {filename:?}"));
    let w = GzEncoder::new(file, Compression::default());
    let mut writer = Writer::from_writer(w);
    for row in rows {
        writer.serialize(row).unwrap();
    }
}

pub const fn extract_time_between(exp: &FileAnswer) -> TimeBetween {
    let time_between_volt = match (&exp.first_volt_warn, &exp.first_volt_err) {
        (Some(w), Some(e)) => Some(e.0 - w.0),
        (None, Some(_)) => Some(0),
        _ => None,
    };

    let time_between_freq = match (&exp.first_freq_warn, &exp.first_freq_err) {
        (Some(w), Some(e)) => Some(e.0 - w.0),
        (None, Some(_)) => Some(0),
        _ => None,
    };

    TimeBetween {
        time_between_volt,
        time_between_freq,
    }
}

pub fn get_context(exp_log_file: &Path) -> Context {
    let mut exp_context_file = exp_log_file.to_path_buf();
    exp_context_file.pop();
    exp_context_file.push("context.json");
    let reader = BufReader::new(
        File::open(&exp_context_file).unwrap_or_else(|_| panic!("Could not get context file of: {exp_log_file:?}")),
    );
    serde_json::from_reader(reader).unwrap()
}

pub fn get_file_answer(exp_log_file: &PathBuf) -> FileAnswer {
    let bfreader = GzDecoder::new(File::open(exp_log_file).unwrap());
    let reader = BufReader::new(bfreader);
    let mut step: i32 = 0;
    let mut warn_err: Vec<GridWarning> = vec![];
    let mut reserve_power: Vec<ReservePower> = vec![];
    let mut frequency: Vec<GridFrequency> = vec![];
    let mut first_freq_warn = None;
    let mut first_freq_err = None;
    let mut first_volt_warn = None;
    let mut first_volt_err = None;
    for (_, line) in reader.lines().enumerate() {
        let l = Line::convert(step, &line.unwrap());
        match l {
            Line::GridInformation { step, content } => {
                frequency.push(GridFrequency::from_value(step, &content));
                reserve_power.push(ReservePower::from_value(step, &content));
            }
            Line::Step { content } => step = content,
            Line::FrequencyWarning { step, content } => {
                if first_freq_warn.is_none() {
                    first_freq_warn = Some((step, content.clone()));
                }
                warn_err.push(GridWarning::from_value(step, &content));
            }
            Line::FrequencyError { step, content } => {
                if first_freq_err.is_none() {
                    first_freq_err = Some((step, content.clone()));
                }
                warn_err.push(GridWarning::from_value(step, &content));
            }
            Line::VoltageWarning { step, content } => {
                if first_volt_warn.is_none() {
                    first_volt_warn = Some((step, content.clone()));
                }
                for c in content {
                    warn_err.push(GridWarning::from_value(step, &c));
                }
            }
            Line::VoltageError { step, content } => {
                if first_volt_err.is_none() {
                    first_volt_err = Some((step, content.clone()));
                }
                for c in content {
                    warn_err.push(GridWarning::from_value(step, &c));
                }
            }
            _ => (),
        }
    }
    FileAnswer {
        first_freq_warn,
        first_freq_err,
        first_volt_warn,
        first_volt_err,
        warn_err,
        reserve_power,
        frequency,
    }
}
