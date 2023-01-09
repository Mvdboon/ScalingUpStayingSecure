use std::fs::{create_dir_all, read_dir, DirEntry};
use std::process::Command;

pub fn extract(from: &str, to: &str, bottom: &Option<i32>, top: &Option<i32>) {
    create_dir_all(to).unwrap();

    let experiments: Vec<DirEntry> = read_dir(from)
        .unwrap()
        .filter_map(|f| f.ok())
        .filter_map(|v| {
            let path = v.path();
            // let contains_u =v.path().file_name().unwrap().to_str().unwrap().contains("_");
            let is_dir = path.is_dir();
            let exp_num = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_end_matches('_')
                .parse::<i32>();

            let in_range = match (bottom, top, exp_num) {
                (Some(l), Some(h), Ok(e)) => e >= *l && e < *h,
                _ => true,
            };

            match (in_range, is_dir) {
                (true, true) => Some(v),
                _ => None,
            }
        })
        .collect();

    for exp in experiments {
        move_folder(exp, to).unwrap();
    }
}

fn move_folder(exp: DirEntry, to: &str) -> Result<std::process::Output, std::io::Error> {
    let command = format!("cp -r {} {}", exp.path().to_string_lossy(), to);
    dbg!(&command);

    #[cfg(target_os = "windows")]
    let x = Command::new("pwsh.exe")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("Could not extract ")
        .wait_with_output();

    #[cfg(target_os = "linux")]
    let x = Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("Could not extract ")
        .wait_with_output();
    x
}
