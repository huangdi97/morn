import os
import base64
import logging
from pathlib import Path
from datetime import datetime, timezone
from typing import Optional

from Crypto.Cipher import AES

logger = logging.getLogger("morn.apz")


class APZStore:
    KEY_SIZE = 32
    NONCE_SIZE = 12
    TAG_SIZE = 16

    def __init__(self, data_dir: Path):
        self.data_dir = Path(data_dir) / "apz"
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self._key = os.urandom(self.KEY_SIZE)
        self._data_file = self.data_dir / "apz_data.enc"
        self._entries: list[dict] = []
        self._load_entries()

    def _load_entries(self):
        if not self._data_file.exists():
            return
        raw = self._data_file.read_text().strip()
        if not raw:
            return
        for line in raw.split("\n"):
            line = line.strip()
            if not line:
                continue
            parts = line.split("|", 2)
            if len(parts) == 3:
                self._entries.append({
                    "id": int(parts[0]),
                    "timestamp": parts[1],
                    "encrypted": parts[2],
                })

    def _save_entries(self):
        lines = [f"{e['id']}|{e['timestamp']}|{e['encrypted']}" for e in self._entries]
        self._data_file.write_text("\n".join(lines) + "\n" if lines else "")

    def _encrypt(self, plaintext: str) -> str:
        nonce = os.urandom(self.NONCE_SIZE)
        cipher = AES.new(self._key, AES.MODE_GCM, nonce=nonce)
        ciphertext, tag = cipher.encrypt_and_digest(plaintext.encode("utf-8"))
        payload = nonce + ciphertext + tag
        return base64.b64encode(payload).decode("ascii")

    def _decrypt(self, encrypted: str) -> str:
        payload = base64.b64decode(encrypted)
        nonce = payload[:self.NONCE_SIZE]
        tag = payload[-self.TAG_SIZE:]
        ciphertext = payload[self.NONCE_SIZE:-self.TAG_SIZE]
        cipher = AES.new(self._key, AES.MODE_GCM, nonce=nonce)
        return cipher.decrypt_and_verify(ciphertext, tag).decode("utf-8")

    def write(self, content: str) -> int:
        entry_id = len(self._entries) + 1
        timestamp = datetime.now(timezone.utc).isoformat()
        encrypted = self._encrypt(content)
        self._entries.append({
            "id": entry_id,
            "timestamp": timestamp,
            "encrypted": encrypted,
        })
        self._save_entries()
        return entry_id

    def read(self, entry_id: int) -> Optional[str]:
        for e in self._entries:
            if e["id"] == entry_id:
                try:
                    return self._decrypt(e["encrypted"])
                except Exception as exc:
                    logger.error("decrypt entry %d failed: %s", entry_id, exc)
                    raise
        return None

    def list_entries(self) -> list[dict]:
        return [
            {"id": e["id"], "timestamp": e["timestamp"]}
            for e in self._entries
        ]

    def count(self) -> int:
        return len(self._entries)
