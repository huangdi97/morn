import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from morn_core.security.rules import SecurityValidator
from morn_core.security.user_protection import UserProtection
from morn.contrib.security_advanced.ethical_judgment import EthicalJudgment
from morn.contrib.security_advanced.risk_guard import RiskGuard


@pytest.fixture
def security_validator():
    return SecurityValidator()


@pytest.fixture
def user_protection():
    return UserProtection()


@pytest.fixture
def ethical_judgment(tmp_path):
    ej = EthicalJudgment(tmp_path)
    ej.enable("active")
    return ej


@pytest.fixture
def risk_guard():
    return RiskGuard()