from pathlib import Path
import sys
import click
from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report, format_text as report_format_text, format_json as report_format_json
from diss_check.calibration import run_calibration, format_text as cal_format_text, format_json as cal_format_json


@click.group(invoke_without_command=True)
@click.pass_context
def main(ctx):
    """Check dissertation PDFs against institutional formatting requirements."""
    if ctx.invoked_subcommand is None:
        click.echo(ctx.get_help())


@main.command()
@click.option("--spec", required=True, type=click.Path(exists=True), help="Path to institution spec YAML file")
@click.option("--json", "output_json", is_flag=True, help="Output results as JSON")
@click.argument("pdf", type=click.Path(exists=True))
def check(spec, pdf, output_json):
    """Check a single dissertation PDF."""
    spec_path = Path(spec)
    pdf_path = Path(pdf)
    try:
        institution_spec = load_spec(spec_path)
    except Exception as e:
        click.echo(f"Error loading spec: {e}", err=True)
        sys.exit(1)
    engine = Engine(institution_spec)
    results = engine.run(pdf_path)
    report = Report(results=results)
    if output_json:
        click.echo(report_format_json(report))
    else:
        click.echo(report_format_text(report))
    if report.summary.fail > 0 or report.summary.error > 0:
        sys.exit(1)


@main.command()
@click.option("--spec", required=True, type=click.Path(exists=True), help="Path to institution spec YAML file")
@click.option("--corpus", required=True, type=click.Path(exists=True, file_okay=False, dir_okay=True), help="Path to corpus directory of accepted dissertations")
@click.option("--json", "output_json", is_flag=True, help="Output results as JSON")
def calibrate(spec, corpus, output_json):
    """Calibrate check suite against a corpus of accepted dissertations."""
    spec_path = Path(spec)
    corpus_path = Path(corpus)
    try:
        institution_spec = load_spec(spec_path)
    except Exception as e:
        click.echo(f"Error loading spec: {e}", err=True)
        sys.exit(1)
    try:
        cal_report = run_calibration(institution_spec, corpus_path, spec_path=spec_path)
    except FileNotFoundError as e:
        click.echo(f"Error: {e}", err=True)
        sys.exit(1)
    if output_json:
        click.echo(cal_format_json(cal_report))
    else:
        click.echo(cal_format_text(cal_report))
    if cal_report.automated_fail_count > 0:
        sys.exit(1)
