import time
import os
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.server import MornState, parse_args, setup_logging


class TestMornState:

    def test_initial_heartbeat_is_zero(self):
        state = MornState()
        assert state.heartbeat_count == 0

    def test_shutdown_defaults_to_false(self):
        state = MornState()
        assert state.shutdown is False

    def test_heartbeat_increment(self):
        state = MornState()
        state.heartbeat_count += 1
        assert state.heartbeat_count == 1
        state.heartbeat_count += 5
        assert state.heartbeat_count == 6

    def test_mem_history_max_length(self):
        state = MornState()
        for i in range(1450):
            state.mem_history.append((time.time(), float(i)))
        assert len(state.mem_history) == 1440

    def test_start_time_is_set_on_creation(self):
        state = MornState()
        now = time.time()
        assert abs(state.start_time - now) < 5

    def test_instance_name_default(self):
        state = MornState()
        assert state.instance_name == "default"

    def test_instance_name_custom(self):
        state = MornState(instance_name="test_morn")
        assert state.instance_name == "test_morn"


class TestParseArgs:

    def test_default_instance(self):
        old_argv = sys.argv
        sys.argv = ["server.py"]
        try:
            args = parse_args()
            assert args.instance == "default"
        finally:
            sys.argv = old_argv

    def test_custom_instance(self):
        old_argv = sys.argv
        sys.argv = ["server.py", "--instance", "my_morn"]
        try:
            args = parse_args()
            assert args.instance == "my_morn"
        finally:
            sys.argv = old_argv


class TestSetupLogging:

    def test_log_file_created(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir)
            logger = setup_logging(data_dir)
            log_file = data_dir / "logs" / "morn.log"
            logger.info("test message")
            assert log_file.exists()
            content = log_file.read_text()
            assert "test message" in content

    def test_multiple_logs_append(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir)
            logger = setup_logging(data_dir)
            logger.info("line1")
            logger.info("line2")
            log_file = data_dir / "logs" / "morn.log"
            lines = log_file.read_text().strip().split("\n")
            assert len(lines) >= 2

    def test_log_format_has_timestamp(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir)
            logger = setup_logging(data_dir)
            logger.info("format_check")
            log_file = data_dir / "logs" / "morn.log"
            content = log_file.read_text()
            assert "T" in content
            assert "|" in content
