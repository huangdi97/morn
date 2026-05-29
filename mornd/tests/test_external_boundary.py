import os
import sys
import json
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.security.external_boundary import ExternalBoundary


class TestInboundRejection:
    def test_reject_all_inbound(self):
        eb = ExternalBoundary(Path(tempfile.mkdtemp()))
        assert not eb.check_inbound("tcp", 80, "192.168.1.1")
        assert not eb.check_inbound("udp", 53, "8.8.8.8")
        assert not eb.check_inbound("http", 443, "10.0.0.1")

    def test_reject_inbound_any_protocol(self):
        eb = ExternalBoundary(Path(tempfile.mkdtemp()))
        assert not eb.check_inbound("ssh", 22, "0.0.0.0")
        assert not eb.check_inbound("https", 443, "::1")


class TestOutboundAllowance:
    def test_register_allowed_outbound(self):
        eb = ExternalBoundary(Path(tempfile.mkdtemp()))
        eb.register_allowed_outbound("ollama", "http://localhost:11434")
        log = eb.get_connection_log()
        assert len(log) == 0

    def test_register_duplicate_outbound_idempotent(self):
        eb = ExternalBoundary(Path(tempfile.mkdtemp()))
        eb.register_allowed_outbound("openai", "https://api.openai.com")
        eb.register_allowed_outbound("openai", "https://api.openai.com")
        log = eb.get_connection_log()
        assert len(log) == 0


class TestPortValidation:
    def test_valid_port(self):
        eb = ExternalBoundary(Path(tempfile.mkdtemp()))
        assert eb.validate_port(443)
        assert eb.validate_port(80)
        assert eb.validate_port(22)

    def test_blocked_port(self):
        d = Path(tempfile.mkdtemp())
        config_dir = d / "security"
        config_dir.mkdir(parents=True, exist_ok=True)
        (config_dir / "external_boundary.json").write_text(
            json.dumps({"blocked_ports": [22, 3389]})
        )
        eb = ExternalBoundary(d)
        assert not eb.validate_port(22)
        assert not eb.validate_port(3389)

    def test_invalid_port_range(self):
        eb = ExternalBoundary(Path(tempfile.mkdtemp()))
        assert not eb.validate_port(-1)
        assert not eb.validate_port(65536)


class TestConnectionLog:
    def test_log_records_inbound_attempts(self):
        eb = ExternalBoundary(Path(tempfile.mkdtemp()))
        eb.check_inbound("tcp", 22, "1.2.3.4")
        eb.check_inbound("udp", 53, "8.8.8.8")
        log = eb.get_connection_log()
        assert len(log) == 2
        assert log[0]["direction"] == "inbound"
        assert log[0]["allowed"] is False

    def test_monitor_disabled_no_log(self):
        d = Path(tempfile.mkdtemp())
        config_dir = d / "security"
        config_dir.mkdir(parents=True, exist_ok=True)
        (config_dir / "external_boundary.json").write_text(
            json.dumps({"monitor_enabled": False})
        )
        eb = ExternalBoundary(d)
        eb.check_inbound("tcp", 80, "1.1.1.1")
        assert len(eb.get_connection_log()) == 0


class TestConfigLoading:
    def test_config_loaded_from_file(self):
        d = Path(tempfile.mkdtemp())
        config_dir = d / "security"
        config_dir.mkdir(parents=True, exist_ok=True)
        (config_dir / "external_boundary.json").write_text(
            json.dumps({"allowed_outbound": [{"service": "test", "endpoint": "http://test"}], "blocked_ports": [8080]})
        )
        eb = ExternalBoundary(d)
        assert not eb.validate_port(8080)
