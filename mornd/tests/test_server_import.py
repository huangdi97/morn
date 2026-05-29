import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))


def test_server_can_import():
    from morn_core.server import MornState, parse_args, setup_logging, main
    assert MornState is not None
    assert parse_args is not None
    assert setup_logging is not None
    assert main is not None


def test_all_submodules_exist():
    import morn_core
    import morn_core.memory
    import morn_core.emotion
    import morn_core.chat
    import morn_core.consciousness
    import morn_core.presence
    import morn_core.security
    assert morn_core.__version__ == "0.1.0"
