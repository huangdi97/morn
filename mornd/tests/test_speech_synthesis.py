import os
import sys
from unittest.mock import patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.media.speech_synthesis import SpeechSynthesizer


class TestSpeechSynthesizer:
    def test_default_disabled(self):
        synth = SpeechSynthesizer()
        assert synth.enabled is False

    def test_speak_raises_when_disabled(self):
        synth = SpeechSynthesizer()
        with pytest.raises(RuntimeError, match="disabled"):
            synth.speak("你好")

    def test_engine_default(self):
        synth = SpeechSynthesizer()
        assert synth._engine == "edge-tts"

    def test_engine_switching(self):
        synth = SpeechSynthesizer()
        synth.set_engine("piper")
        assert synth._engine == "piper"
        synth.set_engine("none")
        assert synth._engine == "none"

    def test_engine_switching_invalid_raises(self):
        synth = SpeechSynthesizer()
        with pytest.raises(ValueError, match="Unsupported engine"):
            synth.set_engine("nonexistent")

    def test_is_available_none_engine(self):
        synth = SpeechSynthesizer()
        synth.set_engine("none")
        assert synth.is_available() is False

    def test_get_available_voices_none_engine(self):
        synth = SpeechSynthesizer()
        synth.set_engine("none")
        voices = synth.get_available_voices()
        assert voices == []

    def test_get_available_voices_edge_tts(self):
        synth = SpeechSynthesizer()
        voices = synth.get_available_voices()
        assert "zh-CN-XiaoxiaoNeural" in voices

    def test_is_available_edge_tts_no_library(self):
        synth = SpeechSynthesizer()
        with patch.dict("sys.modules", {"edge_tts": None}):
            assert synth.is_available() is False

    def test_speak_raises_on_empty_text(self):
        synth = SpeechSynthesizer()
        synth.enabled = True
        with pytest.raises(ValueError, match="cannot be empty"):
            synth.speak("")

    def test_speak_raises_on_none_engine(self):
        synth = SpeechSynthesizer()
        synth.enabled = True
        synth.set_engine("none")
        with pytest.raises(RuntimeError, match="No speech engine"):
            synth.speak("你好")

    def test_speak_raises_when_not_available(self):
        synth = SpeechSynthesizer()
        synth.enabled = True
        with patch.object(synth, "is_available", return_value=False):
            with pytest.raises(RuntimeError, match="not available"):
                synth.speak("你好")