import pytest

from morn.core.security import SecurityValidator, ValidationResult


@pytest.fixture
def validator():
    config = {
        "risk_levels": {},
        "plugin_permissions": {"test_plugin": ["read"]},
    }
    return SecurityValidator(config)


class TestSecurityValidator:
    def test_validate_allowed(self, validator):
        result = validator.validate(
            action_type="read",
            params={"path": "/safe/file.txt"},
            source_plugin="test_plugin",
            risk_level="green",
            risk_preference="green",
        )
        assert result.action == "allow"

    def test_validate_blocked(self, validator):
        result = validator.validate(
            action_type="delete",
            params={"cmd": "rm -rf /"},
            source_plugin="test_plugin",
            risk_level="red",
            risk_preference="green",
        )
        assert result.action == "block"

    def test_validation_result(self, validator):
        result = validator.validate(
            action_type="read",
            params={"path": "/safe/file.txt"},
            source_plugin="test_plugin",
            risk_level="green",
            risk_preference="green",
        )
        assert hasattr(result, "reason")
        assert isinstance(result, ValidationResult)

    def test_different_risk_levels(self, validator):
        r_green = validator.validate(
            action_type="read", params={"path": "/tmp"},
            source_plugin="test_plugin",
            risk_level="green", risk_preference="green",
        )
        assert r_green.action == "allow"

        r_red = validator.validate(
            action_type="read", params={"path": "/tmp"},
            source_plugin="test_plugin",
            risk_level="red", risk_preference="green",
        )
        assert r_red.action == "block"