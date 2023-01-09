use std::fs::{copy, read_dir, rename, DirEntry};
use std::io;
use std::process::{Command, Output};

pub fn run_batch(
    folder: &String,
    dry: &bool,
    compile_binary: &bool,
    low: &Option<i32>,
    high: &Option<i32>,
    log_compress: &bool,
) {
    let file_names = [
        "ModelParameters.ini".to_string(),
        "GridParameters.ini".to_string(),
        "AttackParameters.ini".to_string(),
    ];

    if *compile_binary {
        compile_binary_command();
    }
    copy_binary();

    let mut experiments = get_experiments(folder, low, high);
    experiments.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));
    for exp in experiments {
        println!("Doing experiment {:?}", exp.path().file_name().unwrap());
        if *dry {
            continue;
        }
        move_config_files(&file_names, &exp);
        let exit = run_experiment();
        if *log_compress {
            compress_log(&exp);
        }

        if let Ok(v) = &exit && v.status.success(){
            rename_experiment_folder_to_done(&exp);
            println!("Ran: {:}", exp.path().to_str().unwrap());
        }
        else{
            println!("Failed: {:?}", exp.path())
        };
    }

    cleanup();
}

fn copy_binary() {
    #[cfg(target_os = "windows")]
    let _ = Command::new("pwsh.exe")
        .arg("-c")
        .arg("cp ../SmartGrid_IoT_Security/target/release/modelrunner.exe .")
        .spawn()
        .expect("Could not copy binary")
        .wait();

    #[cfg(target_os = "linux")]
    let _ = Command::new("sh")
        .arg("-c")
        .arg("cp ../SmartGrid_IoT_Security/target/release/modelrunner .")
        .spawn()
        .expect("Could not copy binary")
        .wait();
}

fn compile_binary_command() {
    #[cfg(target_os = "windows")]
    let _ = Command::new("pwsh.exe")
        .arg("-c")
        .arg("cd ../SmartGrid_IoT_Security; cargo build --release")
        .spawn()
        .expect("Could not build")
        .wait();

    #[cfg(target_os = "linux")]
    let _ = Command::new("bash")
        .arg("-c")
        .arg("cd ../SmartGrid_IoT_Security && cargo build --release")
        .spawn()
        .expect("Could not build")
        .wait();
}

pub fn get_experiments(folder: &String, low: &Option<i32>, high: &Option<i32>) -> Vec<DirEntry> {
    let experiments: Vec<DirEntry> = read_dir(folder)
        .unwrap()
        .filter_map(|f| f.ok())
        .filter_map(|v| {
            let path = v.path();
            let is_dir = path.is_dir();
            let exp_num = path.file_name().unwrap().to_str().unwrap().parse::<i32>();

            let in_range = match (low, high, exp_num) {
                (Some(l), Some(h), Ok(e)) => e >= *l && e < *h,
                _ => true,
            };

            let contains_underscore = path.file_name().unwrap().to_string_lossy().contains('_');
            match (contains_underscore, is_dir, in_range) {
                (false, true, true) => Some(v),
                (..) => None,
            }
        })
        .collect();
    experiments
}

fn compress_log(experiment: &DirEntry) {
    let exp_folder_files = read_dir(experiment.path()).unwrap().filter_map(|f| f.ok());
    let logfiles: Vec<DirEntry> = exp_folder_files
        .filter(|s| s.file_name().to_str().unwrap().ends_with(".log"))
        .collect();

    for lf in logfiles {
        let filelocation = format!("gzip {}", lf.path().as_os_str().to_string_lossy());

        #[cfg(target_os = "windows")]
        let _ = Command::new("pwsh.exe")
            .arg("-c")
            .arg(filelocation)
            .spawn()
            .expect("Could not compress ")
            .wait_with_output();

        #[cfg(target_os = "linux")]
        let _ = Command::new("sh")
            .arg("-c")
            .arg(filelocation)
            .spawn()
            .expect("Could not compress ")
            .wait_with_output();
    }
}

fn move_config_files(file_names: &[String; 3], experiment: &DirEntry) {
    for entry in file_names {
        copy(format!("{}/{}", experiment.path().display(), entry), entry)
            .unwrap_or_else(|e| panic!("Copy failed of {}/{entry} due to {e}", experiment.path().display()));
    }
}

fn run_experiment() -> io::Result<Output> {
    #[cfg(target_os = "windows")]
    let x = Command::new("pwsh.exe")
        .arg("-c")
        .arg("./modelrunner.exe")
        .spawn()
        .expect("Unable to run model");

    #[cfg(target_os = "linux")]
    let x = Command::new("bash")
        .arg("-c")
        .arg("./modelrunner")
        .spawn()
        .expect("Unable to run model");
    x.wait_with_output()
}

fn rename_experiment_folder_to_done(experiment: &DirEntry) {
    let from = experiment.path();
    let mut filename = experiment.path().file_name().unwrap().to_owned();
    filename.push("_");
    let to = experiment.path().with_file_name(filename);
    rename(from, to).unwrap();
}

fn cleanup() {
    #[cfg(target_os = "windows")]
    let _ = Command::new("pwsh.exe")
        .arg("-c")
        .arg("rm ./modelrunner.exe")
        .spawn()
        .expect("Unable to remove binary")
        .wait();
    #[cfg(target_os = "linux")]
    let _ = Command::new("bash")
        .arg("-c")
        .arg("rm ./modelrunner")
        .spawn()
        .expect("Unable to remove binary")
        .wait();
}
