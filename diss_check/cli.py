from pathlib import Path
import sys
import click
from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report, format_text, format_json


@click.command()
@click.option("--spec", required=True, type=click.Path(exists=True), help="Path to institution spec YAML file")
@click.option("--json", "output_json", is_flag=True, help="Output results as JSON")
@click.argument("pdf", type=click.Path(exists=True))
def main(spec, pdf, output_json):
    """Check a dissertation PDF against institutional formatting requirements."""
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
        click.echo(format_json(report))
    else:
        click.echo(format_text(report))
    if report.summary.fail > 0 or report.summary.error > 0:
        sys.exit(1)
