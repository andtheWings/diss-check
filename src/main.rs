use std::path::PathBuf;
use std::process;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "diss-check")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Check dissertation PDFs against institutional formatting requirements")]
#[command(long_about = "Automated formatting compliance checker for dissertations and theses.\n\n\
    Reads a YAML spec defining institution-specific formatting rules,\n\
    extracts PDF content with pdf_oxide, and runs automated checkers\n\
    against each requirement. Supports text and JSON output.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run checks against a single dissertation PDF
    Check {
        #[arg(short, long, help = "Path to institution spec YAML file")]
        spec: PathBuf,

        #[arg(short, long, help = "Output results as JSON")]
        json: bool,

        #[arg(short, long, help = "Show only FAIL and ERROR results")]
        quiet: bool,

        #[arg(long, help = "Run only this specific check (by check ID)")]
        check: Option<String>,

        #[arg(short = 'C', long, help = "Run only checks in this category (layout, typography, structure, content)")]
        category: Option<String>,

        #[arg(help = "Path to dissertation PDF")]
        pdf: PathBuf,
    },
    /// Run checks across a corpus of PDFs for calibration
    Calibrate {
        #[arg(short, long, help = "Path to institution spec YAML file")]
        spec: PathBuf,

        #[arg(short, long, help = "Path to corpus directory containing PDF files")]
        corpus: PathBuf,

        #[arg(short, long, help = "Output results as JSON")]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Check { spec, json, quiet, check, category, pdf } => {
            if !pdf.exists() {
                eprintln!("Error: PDF not found: {}", pdf.display());
                process::exit(2);
            }

            let institution_spec = match diss_check::spec::load_spec(spec) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error loading spec: {}", e);
                    process::exit(2);
                }
            };

            let options = diss_check::engine::CheckOptions {
                check_id: check.clone(),
                category: category.clone(),
            };

            let results = match diss_check::engine::run_checks(&institution_spec, pdf, &options) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error running checks: {}", e);
                    process::exit(2);
                }
            };

            let report = diss_check::report::build_report(results);

            if *json {
                match diss_check::report::format_json(&report) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Error formatting JSON: {}", e);
                        process::exit(2);
                    }
                }
            } else if *quiet {
                print!("{}", diss_check::report::format_text_quiet(&report));
            } else {
                println!("{}", diss_check::report::format_text(&report));
            }

            if report.summary.fail > 0 || report.summary.error > 0 {
                process::exit(1);
            }
        }
        Commands::Calibrate { spec, corpus, json } => {
            if !corpus.exists() {
                eprintln!("Error: corpus directory not found: {}", corpus.display());
                process::exit(2);
            }

            let cal_report = match diss_check::calibration::run_calibration(spec, corpus) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(2);
                }
            };

            if *json {
                match diss_check::calibration::format_json(&cal_report) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(2);
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
