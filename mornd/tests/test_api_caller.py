import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.action.api_caller import APICaller


class TestAPICallerValidate:
    def test_private_ip_blocked(self):
        c = APICaller()
        valid, reason = c.validate("https://127.0.0.1/api")
        assert valid is False
        assert "内网" in reason

    def test_localhost_blocked(self):
        c = APICaller()
        valid, reason = c.validate("https://localhost/test")
        assert valid is False

    def test_http_blocked_by_default(self):
        c = APICaller()
        valid, reason = c.validate("http://example.com/api")
        assert valid is False
        assert "非 HTTPS" in reason

    def test_http_allowed_when_configured(self):
        c = APICaller(config={"allow_http": True})
        valid, reason = c.validate("http://example.com/api")
        assert valid is True

    def test_empty_url(self):
        c = APICaller()
        valid, reason = c.validate("")
        assert valid is False

    def test_10_network_blocked(self):
        c = APICaller()
        valid, reason = c.validate("https://10.0.0.1/test")
        assert valid is False
        assert "内网" in reason

    def test_192_168_network_blocked(self):
        c = APICaller()
        valid, reason = c.validate("https://192.168.1.1/admin")
        assert valid is False

    def test_public_https_passes(self):
        c = APICaller()
        valid, reason = c.validate("https://api.github.com")
        assert valid is True


class TestAPICallerCall:
    def test_get_request(self):
        c = APICaller(config={"allow_http": True})
        result = c.call("GET", "http://httpbin.org/get")
        assert result["success"] is True
        assert result["status_code"] == 200

    def test_post_request(self):
        c = APICaller(config={"allow_http": True})
        result = c.call("POST", "http://httpbin.org/post", body={"key": "value"})
        assert result["success"] is True
        assert result["status_code"] == 200

    def test_blocked_request_not_sent(self):
        c = APICaller()
        result = c.call("GET", "https://127.0.0.1/secret")
        assert result["success"] is False
        assert "拦截" in result["error"]

    def test_call_log(self):
        c = APICaller(config={"allow_http": True})
        assert len(c.get_history()) == 0
        c.call("GET", "http://httpbin.org/get")
        assert len(c.get_history()) >= 1


class TestAPICallerMisc:
    def test_is_available(self):
        c = APICaller()
        available = c.is_available()
        assert isinstance(available, bool)