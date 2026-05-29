import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.security.rules import (
    SecurityRule,
    get_all_rules,
    get_rules_by_category,
    add_rule,
    reset_rules,
)


class TestRules:
    def setup_method(self):
        reset_rules()

    def test_all_rules_have_required_fields(self):
        rules = get_all_rules()
        for rule in rules:
            assert isinstance(rule.rule_id, str) and rule.rule_id
            assert isinstance(rule.name, str) and rule.name
            assert isinstance(rule.pattern, str) and rule.pattern
            assert rule.category in ("dangerous", "privacy", "manipulation")
            assert rule.severity in ("critical", "high", "medium")
            assert rule.action_on_match in ("block", "review")
            assert isinstance(rule.description, str) and rule.description

    def test_get_rules_by_category(self):
        dangerous = get_rules_by_category("dangerous")
        assert all(r.category == "dangerous" for r in dangerous)
        assert len(dangerous) > 0

        privacy = get_rules_by_category("privacy")
        assert all(r.category == "privacy" for r in privacy)
        assert len(privacy) > 0

        manipulation = get_rules_by_category("manipulation")
        assert all(r.category == "manipulation" for r in manipulation)
        assert len(manipulation) > 0

    def test_add_rule_dynamically(self):
        count_before = len(get_all_rules())
        new_rule = SecurityRule(
            rule_id="TEST_001",
            name="测试规则",
            pattern="test_pattern",
            category="dangerous",
            severity="medium",
            action_on_match="block",
            description="测试用规则",
        )
        add_rule(new_rule)
        assert len(get_all_rules()) == count_before + 1
        added = [r for r in get_all_rules() if r.rule_id == "TEST_001"]
        assert len(added) == 1
        assert added[0].name == "测试规则"
