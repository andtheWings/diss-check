use std::path::PathBuf;
use std::process;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "diss-check")]
#[command(about = "Check dissertation PDFs against institutional formatting requirements")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Check {
        #[arg(short, long, help = "Path to institution spec YAML file")]
        spec: PathBuf,

        #[arg(short, long, help = "Output results as JSON")]
        json: bool,

        #[arg(help = "Path to dissertation PDF")]
        pdf: PathBuf,
    },
    Calibrate {
        #[arg(short, long, help = "Path to institution spec YAML file")]
        spec: PathBuf,

        #[arg(short, long, help = "Path to corpus directory")]
        corpus: PathBuf,

        #[arg(short, long, help = "Output results as JSON")]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Check { spec, json, pdf } => {
            let institution_spec = match diss_check::spec::load_spec(spec) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error loading spec: {}", e);
                    process::exit(1);
                }
            };

            let results = match diss_check::engine::run_checks(&institution_spec, pdf) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error running checks: {}", e);
                    process::exit(1);
                }
            };

            let report = diss_check::report::build_report(results);

            if *json {
                match diss_check::report::format_json(&report) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Error formatting JSON: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                println!("{}", diss_check::report::format_text(&report));
            }

            if report.summary.fail > 0 || report.summary.error > 0 {
                process::exit(1);
            }
        }
        Commands::Calibrate { spec, corpus, json } => {
            let cal_report = match diss_check::calibration::run_calibration(spec, corpus) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };

            if *json {
                match diss_check::calibration::format_json(&cal_report) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                println!("{}", diss_check::calibration::format_text(&cal_report));
            }

            if cal_report.automated_fail_count() > 0 {
                process::exit(1);
            }
        }
    }
}
