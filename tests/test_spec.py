from diss_check.spec import load_spec


def test_load_minimal_iu_spec():
    spec = load_spec("specs/iu.yaml")
    assert spec.institution == "Indiana University"
    assert spec.source_revision == "September 2025"
    assert len(spec.checks) == 5
    checkers = {c.checker for c in spec.checks}
    assert checkers >= {"margins", "font_size", "font_weight", "font_family", "section_presence"}
    assert spec.checks[0].automatable is True
    assert spec.checks[4].checker == "section_presence"
    assert spec.checks[4].automatable is False


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
