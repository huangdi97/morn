import os
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.action.cli_executor import CLIExecutor


class TestCLIExecutorValidate:
    def test_safe_command_pass_validation(self):
        e = CLIExecutor()
        valid, reason = e.validate("echo hello")
        assert valid is True
        assert reason == ""

    def test_dangerous_rm_rf_blocked(self):
        e = CLIExecutor()
        valid, reason = e.validate("rm -rf /")
        assert valid is False
        assert "危险命令" in reason

    def test_dangerous_dd_blocked(self):
        e = CLIExecutor()
        valid, reason = e.validate("dd if=/dev/zero of=/dev/sda")
        assert valid is False

    def test_fork_bomb_blocked(self):
        e = CLIExecutor()
        valid, reason = e.validate(":(){ :|:& };:")
        assert valid is False

    def test_chmod_777_blocked(self):
        e = CLIExecutor()
        valid, reason = e.validate("chmod 777 /")
        assert valid is False

    def test_sudo_blocked(self):
        e = CLIExecutor()
        valid, reason = e.validate("sudo apt install")
        assert valid is False


class TestCLIExecutorExecute:
    def test_execute_simple_command(self):
        e = CLIExecutor()
        result = e.execute("echo hello")
        assert result["success"] is True
        assert "hello" in result.get("stdout", "")

    def test_execute_failing_command(self):
        e = CLIExecutor()
        result = e.execute("exit 42")
        assert result["success"] is False
        assert result["returncode"] == 42

    def test_execute_blocks_dangerous(self):
        e = CLIExecutor()
        result = e.execute("rm -rf /")
        assert result["success"] is False
        assert "拦截" in result["error"]

    def test_execute_timeout(self):
        e = CLIExecutor()
        start = time.time()
        result = e.execute("sleep 10", timeout=1)
        elapsed = time.time() - start
        assert result["success"] is False
        assert "超时" in result["error"]
        assert elapsed < 5

    def test_execute_with_workdir(self):
        e = CLIExecutor()
        result = e.execute("pwd", workdir="/tmp")
        assert result["success"] is True
        assert "/tmp" in result.get("stdout", "")


class TestCLIExecutorMisc:
    def test_is_available(self):
        e = CLIExecutor()
        assert e.is_available() is True

    def test_get_shell(self):
        e = CLIExecutor()
        shell = e.get_shell()
        assert isinstance(shell, str)
        assert len(shell) > 0

    def test_execution_log(self):
        e = CLIExecutor()
        assert len(e.get_history()) == 0
        e.execute("echo log_test")
        e.execute("echo log_test2")
        assert len(e.get_history()) == 2
        assert e.get_history()[0]["command"] == "echo log_test"

    def test_empty_command_validation(self):
        e = CLIExecutor()
        valid, reason = e.validate("")
        assert valid is False
        assert "空命令" in reason