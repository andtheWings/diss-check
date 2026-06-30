from diss_check.spec import load_spec


def test_load_minimal_iu_spec():
    spec = load_spec("specs/iu.yaml")
    assert spec.institution == "Indiana University"
    assert spec.source_revision == "September 2025"
    assert len(spec.checks) == 2
    assert spec.checks[0].checker == "margins"
    assert spec.checks[0].automatable is True
    assert spec.checks[1].automatable is False  # structure check is manual for now


def test_spec_validates_invalid_yaml(tmp_path):
    yaml_content = """
institution: Test
source_revision: "v1"
checks:
  - id: bad
    category: invalid_category
    checker: x
    target: {}
    params: {}
"""
    spec_file = tmp_path / "bad.yaml"
    spec_file.write_text(yaml_content)
    import pytest
    with pytest.raises(Exception):
        load_spec(spec_file)
