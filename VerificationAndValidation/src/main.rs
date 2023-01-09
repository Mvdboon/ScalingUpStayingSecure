#![allow(unused)]
mod experiments;
mod statistics;
mod validation;

use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Validate {
        #[arg(short, long, value_name = "Download the required files from the tennet website")]
        download: bool,
        #[arg(short, long, value_name = "Validation folder for data input and output")]
        folder:   Option<String>,
    },
    Experiments {
        #[arg(short, long, value_name = "Folder where the result files are located")]
        experiments_folder: Option<String>,
        #[arg(short, long, value_name = "Folder where data-analysis will take place")]
        folder:             Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Validate { download, folder } => {
                let f = folder.unwrap_or("validation".to_string());
                validation::validate(&download, &f)
            }
            Commands::Experiments {
                experiments_folder,
                folder,
            } => {
                let ef = experiments_folder.unwrap_or("../Experiments/experiments".to_string());
                let df = folder.unwrap_or("experiments".to_string());
                experiments::analyse(ef, df);
            }
        }
    } else {
        panic!("Need to input a command");
    };
}
