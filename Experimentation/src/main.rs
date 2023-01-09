#![warn(
// clippy::cargo,
clippy::complexity,
clippy::correctness,
clippy::nursery,
clippy::perf,
clippy::style,
clippy::suspicious,
clippy::todo,
// clippy::pedantic,
// clippy::unwrap_used,
// clippy::expect_used,
)]
#![feature(let_chains, is_some_and)]

mod creator;
mod extract;
#[allow(clippy::fallible_impl_from)]
mod parse;
mod run;


use crate::creator::create_experiments;
use crate::parse::parse_experiments;
use crate::run::run_batch;

use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    //     #[arg(short, long)]
    //     batch:   Option<String>,

    //     #[arg(short, long)]
    //     create: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create the experiments
    Create {
        // #[arg(short, long, value_name = "FOLDER")]
        folder: String,
    },
    Run {
        // #[arg(short, long, value_name = "Folder")]
        folder:         String,
        #[arg(short, long, value_name = "Experiment low")]
        bottom:         Option<i32>,
        #[arg(short, long, value_name = "Experiment how")]
        top:            Option<i32>,
        #[arg(short, long, value_name = "Dry Run")]
        dry:            bool,
        #[arg(short, long, value_name = "compile_binary")]
        compile_binary: bool,
        #[arg(short, long, value_name = "log_compressing")]
        log_compress:   bool,
    },
    Extract {
        // #[arg(short, long, value_name = "Folder")]
        from:   String,
        to:     String,
        #[arg(short, long, value_name = "Experiment low")]
        bottom: Option<i32>,
        #[arg(short, long, value_name = "Experiment how")]
        top:    Option<i32>,
    },
    Parse {
        folder: String,
        #[arg(short, long, value_name = "Experiment low")]
        bottom: Option<i32>,
        #[arg(short, long, value_name = "Experiment how")]
        top:    Option<i32>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Create { folder }) => create_experiments(folder),
        Some(Commands::Run {
            folder,
            dry,
            compile_binary,
            bottom,
            top,
            log_compress,
        }) => run_batch(folder, dry, compile_binary, bottom, top, log_compress),
        Some(Commands::Extract { from, to, bottom, top }) => extract::extract(from, to, bottom, top),
        Some(Commands::Parse { folder, bottom, top }) => parse_experiments(folder, bottom, top),
        None => (),
    };
}
