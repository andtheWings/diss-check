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
    }
}
