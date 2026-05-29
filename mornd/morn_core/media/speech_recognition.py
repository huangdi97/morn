import logging
from pathlib import Path

logger = logging.getLogger("morn.speech.recognition")

_SUPPORTED_LANGUAGES = {
    "zh-cn": "Chinese (Simplified)",
    "en-us": "English (US)",
    "ja": "Japanese",
    "ko": "Korean",
    "fr": "French",
    "de": "German",
    "ru": "Russian",
}

_DEFAULT_CONFIG = {
    "enabled": False,
    "model_path": str(Path.home() / ".morn" / "models" / "vosk"),
    "language": "zh-cn",
}


class SpeechRecognizer:
    def __init__(self, config: dict = None):
        self._config = dict(_DEFAULT_CONFIG)
        if config:
            self._config.update({k: v for k, v in config.items() if k in _DEFAULT_CONFIG})
        self._vosk = None

    @property
    def enabled(self) -> bool:
        return self._config.get("enabled", False)

    @enabled.setter
    def enabled(self, value: bool):
        self._config["enabled"] = value

    @property
    def language(self) -> str:
        return self._config.get("language", "zh-cn")

    @language.setter
    def language(self, value: str):
        if value in _SUPPORTED_LANGUAGES:
            self._config["language"] = value

    @property
    def model_path(self) -> Path:
        return Path(self._config.get("model_path", ""))

    def is_available(self) -> bool:
        try:
            model_dir = self.model_path
            if not model_dir.exists():
                return False
            lang_dir = model_dir / self.language
            if lang_dir.exists() and any(lang_dir.iterdir()):
                return True
            return any(model_dir.iterdir()) if model_dir.exists() else False
        except Exception:
            return False

    def get_languages(self) -> dict:
        return dict(_SUPPORTED_LANGUAGES)

    def download_model(self, lang: str = None) -> str:
        target_lang = lang or self.language
        if target_lang not in _SUPPORTED_LANGUAGES:
            raise ValueError(f"Unsupported language: {target_lang}. Supported: {list(_SUPPORTED_LANGUAGES.keys())}")
        if not self.enabled:
            raise RuntimeError("Speech recognition is disabled. Enable it before downloading models.")
        logger.info("download_model called for %s — Vosk model download requires network access", target_lang)
        raise NotImplementedError(
            f"Automatic Vosk model download is not implemented. "
            f"Please manually download the {target_lang} model to {self.model_path / target_lang}"
        )

    def transcribe(self, audio_path: str) -> str:
        if not self.enabled:
            raise RuntimeError("Speech recognition is disabled")
        if not self.is_available():
            raise RuntimeError(
                f"Vosk model not found at {self.model_path / self.language}. "
                f"Download a model first or set a valid model_path."
            )
        audio_path_obj = Path(audio_path)
        if not audio_path_obj.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")
        raise NotImplementedError(
            "Vosk transcription requires the vosk library. "
            "Install with: pip install vosk"
        )
