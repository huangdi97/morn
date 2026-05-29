import os
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import pytest
from morn.contrib.security_advanced.apz_store import APZStore


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_apz_test_") as tmpdir:
        yield Path(tmpdir)


class TestAPZStoreEncryption:

    def test_encrypt_decrypt_roundtrip(self, data_dir):
        store = APZStore(data_dir)
        store.write("Hello, this is a secret message!")
        assert store.read(1) == "Hello, this is a secret message!"

    def test_encrypted_file_not_plaintext(self, data_dir):
        store = APZStore(data_dir)
        store.write("this is secret")
        raw = (data_dir / "apz" / "apz_data.enc").read_text()
        assert "secret" not in raw

    def test_different_key_cannot_decrypt(self, data_dir):
        store1 = APZStore(data_dir)
        store1.write("secret data")
        store2 = APZStore(data_dir)
        with pytest.raises((ValueError, Exception)):
            store2.read(1)

    def test_chinese_content(self, data_dir):
        store = APZStore(data_dir)
        store.write("你好，这是一个绝对隐私的测试消息！")
        assert store.read(1) == "你好，这是一个绝对隐私的测试消息！"

    def test_empty_content(self, data_dir):
        store = APZStore(data_dir)
        store.write("")
        assert store.read(1) == ""

    def test_long_content(self, data_dir):
        store = APZStore(data_dir)
        plain = "隐私数据" * 1024
        store.write(plain)
        assert store.read(1) == plain


class TestAPZStorePersistence:

    def test_write_and_count(self, data_dir):
        store = APZStore(data_dir)
        assert store.count() == 0
        store.write("first entry")
        assert store.count() == 1

    def test_multiple_writes(self, data_dir):
        store = APZStore(data_dir)
        for i in range(5):
            store.write(f"entry {i}")
        assert store.count() == 5
        for i in range(5):
            assert store.read(i + 1) == f"entry {i}"

    def test_read_nonexistent_returns_none(self, data_dir):
        store = APZStore(data_dir)
        assert store.read(999) is None

    def test_list_entries_returns_metadata(self, data_dir):
        store = APZStore(data_dir)
        store.write("alpha")
        store.write("beta")
        entries = store.list_entries()
        assert len(entries) == 2
        assert entries[0]["id"] == 1
        assert entries[1]["id"] == 2
        assert "timestamp" in entries[0]
        assert "encrypted" not in entries[0]

    def test_apz_directory_created(self, data_dir):
        APZStore(data_dir)
        assert (data_dir / "apz").is_dir()

    def test_data_file_created_after_write(self, data_dir):
        store = APZStore(data_dir)
        assert not (data_dir / "apz" / "apz_data.enc").exists()
        store.write("test")
        assert (data_dir / "apz" / "apz_data.enc").exists()


class TestAPZStoreSecurityBoundary:

    def test_key_not_persisted_to_disk(self, data_dir):
        store = APZStore(data_dir)
        store.write("secret")
        apz_dir = data_dir / "apz"
        files = list(apz_dir.iterdir())
        for f in files:
            content = f.read_bytes()
            assert store._key not in content

    def test_no_read_all_method(self, data_dir):
        store = APZStore(data_dir)
        assert not hasattr(store, "read_all")

    def test_no_decrypt_all_method(self, data_dir):
        store = APZStore(data_dir)
        assert not hasattr(store, "decrypt_all")

    def test_new_process_cannot_read_old_data(self, data_dir):
        store1 = APZStore(data_dir)
        store1.write("old process secret")
        store2 = APZStore(data_dir)
        with pytest.raises((ValueError, Exception)):
            store2.read(1)
