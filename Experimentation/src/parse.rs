use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

mod functions;
mod structs;

use functions::*;
use structs::*;

pub fn parse_experiments(folder: &str, _bottom: &Option<i32>, _top: &Option<i32>) {
    let files = glob::glob(&format!("{folder}/**/*.log.gz")).unwrap();
    let files: Vec<PathBuf> = files.map(|f| f.unwrap()).collect();
    let pg_len = files.len();
    let _parsed_experiments: Vec<(PathBuf, ExperimentResult)> = files
        .into_par_iter()
        .progress_count(pg_len as u64)
        .map(|f| {
            let mut exp_folder = f.clone();
            exp_folder.pop();
            let (file_answer, experiment_result) = parse_experiment(f.clone());
            output_experiment(&mut exp_folder, &file_answer, &experiment_result);
            (f, experiment_result)
        })
        .collect();
}

fn parse_experiment(exp_log_file: PathBuf) -> (FileAnswer, ExperimentResult) {
    let file_answer = get_file_answer(&exp_log_file);
    let context = get_context(&exp_log_file);
    let experiment_result = ExperimentResult::create(context, &file_answer);
    (file_answer, experiment_result)
}

fn output_experiment(exp_folder: &mut PathBuf, fa: &FileAnswer, er: &ExperimentResult) {
    let file = exp_folder;

    file.push("reg_room.csv.gz");
    let reg_room = &fa.reserve_power;
    output_csv_gz(file, reg_room);
    file.pop();

    file.push("grid_warning_error.csv.gz");
    let gridwarning = &fa.warn_err;
    output_csv_gz(file, gridwarning);
    file.pop();

    file.push("grid_frequency.csv.gz");
    let grid_freq = &fa.frequency;
    output_csv_gz(file, grid_freq);
    file.pop();

    let folder_name = file.file_name().unwrap().to_str().unwrap();
    file.push(format!("{folder_name}.result"));
    let param_file = BufWriter::new(File::create(&file).unwrap());
    serde_json::to_writer_pretty(param_file, &er).unwrap();
    file.pop();
}
