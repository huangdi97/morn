from morn.core.sandbox import Sandbox, SandboxLevel, get_sandbox_for


class TestSandbox:
    def test_sandbox_levels(self):
        assert SandboxLevel.SAME == "same_process"
        assert SandboxLevel.SECCOMP == "seccomp"
        assert SandboxLevel.NSJAIL == "nsjail"
        assert SandboxLevel.FIRECRACKER == "firecracker"

    def test_get_sandbox_for(self):
        s = get_sandbox_for("A")
        assert s is not None
        assert s.level == "seccomp"

    def test_sandbox_properties(self):
        s = Sandbox("seccomp", allowed_paths=["/tmp"])
        assert s.level == "seccomp"
        assert s.allowed_paths == ["/tmp"]