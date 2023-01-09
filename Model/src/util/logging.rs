use std::fs::create_dir_all;

use fast_log::appender::{Command, RecordFormat};
use fast_log::Config;
pub use log::{debug, error, info, trace, warn, LevelFilter};

use crate::model::ModelParameters;
use crate::util::UtilError;

/// Initializing the logger.
pub fn init(param: &ModelParameters) -> Result<(), UtilError> {
    create_output_folder(param)?;
    fast_log::init(
        Config::new()
            .level(param.loglevel)
            .format(LogFormat::new())
            .file(&format!("{}/{}.log", param.logfolder, param.logfile)),
    )?;
    Ok(())
}

fn create_output_folder(param: &ModelParameters) -> Result<(), UtilError> {
    let mut folders = vec![&param.logfolder];
    if param.enable_output {
        folders.push(&param.outputdatafolder);
    }
    for folder in folders {
        if create_dir_all(folder).is_ok() {
            debug!("{} - Folder was created: {}", param.name, folder);
        } else {
            let dbg_str = format!("{} - Folder already excisted: {folder}", param.name);
            debug!("{dbg_str}");
            return Err(UtilError::FileSystemError(dbg_str));
        }
    }
    Ok(())
}

struct LogFormat {}

impl LogFormat {
    pub(crate) fn new() -> Self { Self {} }
}

impl Default for LogFormat {
    fn default() -> Self { Self::new() }
}

impl RecordFormat for LogFormat {
    fn do_format(&self, arg: &mut fast_log::appender::FastLogRecord) {
        match &arg.command {
            Command::CommandRecord => {
                let now = fastdate::DateTime::now();
                let mut time_str = now.to_string();
                if time_str.len() < 22 {
                    time_str.push('0');
                }
                arg.formated = format!(
                    "{} - {:5} - {}\n",
                    if time_str.len() > 22 {
                        &time_str[..22]
                    } else {
                        &time_str
                    },
                    arg.level,
                    arg.args
                );
            }
            Command::CommandExit => {}
            Command::CommandFlush(_) => {}
        }
    }
}
