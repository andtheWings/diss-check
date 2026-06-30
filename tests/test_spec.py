from diss_check.spec import load_spec


def test_load_minimal_iu_spec():
    spec = load_spec("specs/iu.yaml")
    assert spec.institution == "Indiana University"
    assert spec.source_revision == "September 2025"
    assert len(spec.checks) == 11
    checkers = {c.checker for c in spec.checks}
    assert checkers >= {"margins", "font_size", "font_weight", "font_family", "justification",
                        "section_presence", "section_order", "boilerplate_match", "committee_order"}
    assert spec.checks[-1].checker == "review"
    assert spec.checks[-1].automatable is True


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
