import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.media.speech_recognition import SpeechRecognizer


class TestSpeechRecognizer:
    def test_default_disabled(self):
        recognizer = SpeechRecognizer()
        assert recognizer.enabled is False

    def test_transcribe_raises_when_disabled(self):
        recognizer = SpeechRecognizer()
        with pytest.raises(RuntimeError, match="disabled"):
            recognizer.transcribe("test.wav")

    def test_is_available_returns_false_without_model(self):
        recognizer = SpeechRecognizer()
        assert recognizer.is_available() is False

    def test_get_languages_returns_expected_list(self):
        recognizer = SpeechRecognizer()
        langs = recognizer.get_languages()
        assert "zh-cn" in langs
        assert "en-us" in langs
        assert "ja" in langs

    def test_language_switching(self):
        recognizer = SpeechRecognizer()
        assert recognizer.language == "zh-cn"
        recognizer.language = "en-us"
        assert recognizer.language == "en-us"

    def test_language_switching_invalid_keeps_default(self):
        recognizer = SpeechRecognizer()
        recognizer.language = "invalid-lang"
        assert recognizer.language == "zh-cn"

    def test_download_model_raises_when_disabled(self):
        recognizer = SpeechRecognizer()
        with pytest.raises(RuntimeError, match="disabled"):
            recognizer.download_model()

    def test_download_model_raises_not_implemented_when_enabled(self):
        recognizer = SpeechRecognizer()
        recognizer.enabled = True
        with pytest.raises(NotImplementedError, match="not implemented"):
            recognizer.download_model()

    def test_download_model_unsupported_language(self):
        recognizer = SpeechRecognizer()
        recognizer.enabled = True
        with pytest.raises(ValueError, match="Unsupported language"):
            recognizer.download_model(lang="xx")

    def test_transcribe_raises_on_missing_file(self):
        recognizer = SpeechRecognizer()
        recognizer.enabled = True
        with patch.object(recognizer, "is_available", return_value=True):
            with pytest.raises(FileNotFoundError, match="not found"):
                recognizer.transcribe("/nonexistent/file.wav")

    def test_model_config_custom_path(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            recognizer = SpeechRecognizer({"model_path": tmpdir, "enabled": True})
            assert recognizer.model_path == Path(tmpdir)
            assert recognizer.enabled is True

    def test_is_available_true_when_model_exists(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            lang_dir = Path(tmpdir) / "zh-cn"
            lang_dir.mkdir(parents=True)
            (lang_dir / "model.conf").touch()
            recognizer = SpeechRecognizer({"model_path": tmpdir, "language": "zh-cn"})
            assert recognizer.is_available() is True