from pathlib import Path
import pytest


@pytest.fixture
def fixtures_dir():
    return Path(__file__).parent / "fixtures"


@pytest.fixture
def test_dissertation_path(fixtures_dir):
    return fixtures_dir / "2020-12-chambers.pdf"


@pytest.fixture
def iu_template_path(fixtures_dir):
    return fixtures_dir / "iu_template.pdf"
