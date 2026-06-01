import pytest

from morn.core.resource_quota import TokenCounter, QuotaManager, QuotaExceeded


class TestResourceQuota:
    def test_token_counter(self):
        counter = TokenCounter()
        count = counter.count_input("hello world")
        assert count >= 0

    def test_quota_check_allow(self):
        qm = QuotaManager(global_budget=1000, global_period=60)
        assert qm.check("S", 10, "test") is True

    def test_quota_exceeded(self):
        qm = QuotaManager(global_budget=100, global_period=60)
        qm.check("S", 200, "test")
        with pytest.raises(QuotaExceeded):
            qm.consume("S", 200, "test")