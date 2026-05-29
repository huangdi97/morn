import logging
from pathlib import Path

logger = logging.getLogger("morn.speech.synthesis")

_SUPPORTED_ENGINES = ["edge-tts", "piper", "none"]

_DEFAULT_VOICES = {
    "edge-tts": [
        "zh-CN-XiaoxiaoNeural",
        "zh-CN-YunxiNeural",
        "zh-CN-YunyangNeural",
        "en-US-JennyNeural",
        "en-US-GuyNeural",
        "ja-JP-NanamiNeural",
        "ko-KR-SunHiNeural",
    ],
    "piper": [],
}

_DEFAULT_CONFIG = {
    "enabled": False,
    "engine": "edge-tts",
    "voice": "zh-CN-XiaoxiaoNeural",
    "output_dir": str(Path.home() / ".morn" / "speech_output"),
}


class SpeechSynthesizer:
    def __init__(self, config: dict = None):
        self._config = dict(_DEFAULT_CONFIG)
        if config:
            self._config.update({k: v for k, v in config.items() if k in _DEFAULT_CONFIG})
        self._engine = self._config.get("engine", "edge-tts")

    @property
    def enabled(self) -> bool:
        return self._config.get("enabled", False)

    @enabled.setter
    def enabled(self, value: bool):
        self._config["enabled"] = value

    def is_available(self) -> bool:
        if self._engine == "none":
            return False
        if self._engine == "edge-tts":
            try:
                import edge_tts
                return True
            except ImportError:
                return False
        if self._engine == "piper":
            piper_path = Path("/usr/bin/piper") or Path("/usr/local/bin/piper")
            return piper_path.exists()
        return False

    def get_available_voices(self) -> list:
        if self._engine == "none":
            return []
        if self._engine == "edge-tts":
            try:
                import edge_tts
                return list(_DEFAULT_VOICES.get("edge-tts", []))
            except ImportError:
                return list(_DEFAULT_VOICES.get("edge-tts", []))
        if self._engine == "piper":
            return list(_DEFAULT_VOICES.get("piper", []))
        return []

    def set_engine(self, engine: str):
        if engine not in _SUPPORTED_ENGINES:
            raise ValueError(f"Unsupported engine: {engine}. Supported: {_SUPPORTED_ENGINES}")
        self._engine = engine
        self._config["engine"] = engine

    def speak(self, text: str, voice: str = None) -> str:
        if not self.enabled:
            raise RuntimeError("Speech synthesis is disabled")
        if self._engine == "none":
            raise RuntimeError("No speech engine configured")
        if not text or not text.strip():
            raise ValueError("Text to speak cannot be empty")
        if not self.is_available():
            raise RuntimeError(
                f"Speech engine '{self._engine}' is not available. "
                f"Install the required package or switch to a different engine."
            )
        output_dir = Path(self._config.get("output_dir", ""))
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"speech_{abs(hash(text)) % 10**8}.mp3"
        raise NotImplementedError(
            f"Speech synthesis via '{self._engine}' requires the engine library. "
            f"Install with: pip install edge-tts"
        )
