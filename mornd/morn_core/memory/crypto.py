import os
import json
import base64
import logging
from pathlib import Path

from Crypto.Cipher import AES
from Crypto.Protocol.KDF import PBKDF2


class MemoryCrypto:
    KEY_SIZE = 32
    SALT_SIZE = 16
    NONCE_SIZE = 12
    TAG_SIZE = 16

    def __init__(self, data_dir: Path):
        self.data_dir = Path(data_dir)
        self._key = None
        self._load_or_create_key()

    def _load_or_create_key(self):
        key_path = self.data_dir / "memory.key"
        if key_path.exists():
            raw = key_path.read_bytes()
            if len(raw) == self.KEY_SIZE:
                self._key = raw
            else:
                try:
                    self._key = base64.b64decode(raw.strip())
                    if len(self._key) != self.KEY_SIZE:
                        raise ValueError(f"key size mismatch: {len(self._key)}")
                except Exception:
                    raise RuntimeError("memory.key format error")
        else:
            self._key = os.urandom(self.KEY_SIZE)
            key_path.write_bytes(self._key)
            key_path.chmod(0o600)

    def encrypt(self, plaintext: str) -> str:
        if not plaintext:
            return plaintext
        nonce = os.urandom(self.NONCE_SIZE)
        cipher = AES.new(self._key, AES.MODE_GCM, nonce=nonce)
        ciphertext, tag = cipher.encrypt_and_digest(plaintext.encode("utf-8"))
        payload = nonce + ciphertext + tag
        return "ENC:" + base64.b64encode(payload).decode("ascii")

    def decrypt(self, encrypted: str) -> str:
        if not encrypted:
            return encrypted
        if not encrypted.startswith("ENC:"):
            return encrypted
        try:
            payload = base64.b64decode(encrypted[4:])
            nonce = payload[:self.NONCE_SIZE]
            tag = payload[-self.TAG_SIZE:]
            ciphertext = payload[self.NONCE_SIZE:-self.TAG_SIZE]
            cipher = AES.new(self._key, AES.MODE_GCM, nonce=nonce)
            return cipher.decrypt_and_verify(ciphertext, tag).decode("utf-8")
        except Exception as e:
            logger = logging.getLogger("morn.crypto")
            logger.error(f"decrypt failed: {e}")
            return encrypted

    def is_encrypted(self, text: str) -> bool:
        return bool(text) and text.startswith("ENC:")
