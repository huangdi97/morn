import json
import os
import stat
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.crypto import MemoryCrypto
from morn_core.memory.store import MemoryStore


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir)


class TestMemoryCrypto:

    @pytest.fixture
    def crypto(self, data_dir):
        return MemoryCrypto(data_dir)

    def test_encrypt_decrypt_roundtrip(self, crypto):
        plain = "Hello, this is a secret message!"
        encrypted = crypto.encrypt(plain)
        assert encrypted != plain
        assert encrypted.startswith("ENC:")
        decrypted = crypto.decrypt(encrypted)
        assert decrypted == plain

    def test_key_generation(self, data_dir):
        key_path = data_dir / "memory.key"
        assert not key_path.exists()
        MemoryCrypto(data_dir)
        assert key_path.exists()
        raw = key_path.read_bytes()
        assert len(raw) == 32

    def test_key_persistence(self, data_dir):
        c1 = MemoryCrypto(data_dir)
        c2 = MemoryCrypto(data_dir)
        assert c1._key == c2._key

    def test_decrypt_unencrypted_text(self, crypto):
        plain = "This is not encrypted"
        result = crypto.decrypt(plain)
        assert result == plain

    def test_empty_string(self, crypto):
        assert crypto.encrypt("") == ""
        assert crypto.decrypt("") == ""

    def test_encrypted_prefix_detection(self, crypto):
        assert crypto.is_encrypted("ENC:abc123") is True
        assert crypto.is_encrypted("plain text") is False
        assert crypto.is_encrypted("") is False
        assert crypto.is_encrypted("ENC:") is True

    def test_encrypt_chinese(self, crypto):
        plain = "你好，这是一个测试消息！创建者今天心情很好。"
        encrypted = crypto.encrypt(plain)
        assert encrypted.startswith("ENC:")
        decrypted = crypto.decrypt(encrypted)
        assert decrypted == plain

    def test_long_text(self, crypto):
        plain = "你好，" * 1024
        encrypted = crypto.encrypt(plain)
        decrypted = crypto.decrypt(encrypted)
        assert decrypted == plain

    def test_invalid_ciphertext_returns_original(self, crypto):
        result = crypto.decrypt("ENC:this_is_not_valid_base64!!!")
        assert result == "ENC:this_is_not_valid_base64!!!"

    def test_key_file_permissions(self, data_dir):
        MemoryCrypto(data_dir)
        key_path = data_dir / "memory.key"
        st = os.stat(key_path)
        mode = stat.S_IMODE(st.st_mode)
        assert mode == 0o600


@pytest.mark.asyncio
class TestMemoryStoreWithEncryption:

    async def test_integration_with_store_write(self, data_dir):
        async with MemoryStore(data_dir, enable_encryption=True) as store:
            event_id = await store.add_capsule({
                "entities": json.dumps(["创建者"]),
                "description": "创建者说今天心情很好",
            })
            cursor = await store.db.execute(
                "SELECT description FROM capsules WHERE event_id = ?", (event_id,))
            row = await cursor.fetchone()
            assert row["description"].startswith("ENC:")

    async def test_integration_with_store_read(self, data_dir):
        async with MemoryStore(data_dir, enable_encryption=True) as store:
            event_id = await store.add_capsule({
                "entities": json.dumps(["创建者"]),
                "description": "创建者说今天心情很好",
            })
            cap = await store.get_capsule(event_id)
            assert cap is not None
            assert cap["description"] == "创建者说今天心情很好"

    async def test_search_fts_with_encryption(self, data_dir):
        async with MemoryStore(data_dir, enable_encryption=True) as store:
            await store.add_capsule({
                "entities": json.dumps(["创建者"]),
                "description": "创建者提到他正在研究衰老干预",
            })
            await store.add_capsule({
                "entities": json.dumps(["创建者"]),
                "description": "今天天气很好，适合散步",
            })
            results = await store.search_fts("衰老")
            assert len(results) >= 1
            assert "衰老" in results[0]["description"]

    async def test_get_recent_with_encryption(self, data_dir):
        async with MemoryStore(data_dir, enable_encryption=True) as store:
            await store.add_capsule({"entities": "[]", "description": "first"})
            import asyncio
            await asyncio.sleep(0.01)
            await store.add_capsule({"entities": "[]", "description": "second"})
            recent = await store.get_recent(limit=2)
            assert recent[0]["description"] == "second"
