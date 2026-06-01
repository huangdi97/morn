import os
import sys
import time


sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import RedisCache


class TestRedisCache:
    def test_connect_disconnect(self):
        cache = RedisCache()
        assert cache.connect() is True
        assert cache.disconnect() is True

    def test_is_connected_when_disabled(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        assert cache.is_connected() is False

    def test_set_get(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        cache.set("key1", "value1")
        assert cache.get("key1") == "value1"

    def test_get_nonexistent(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        assert cache.get("nonexistent") is None

    def test_set_get_overwrite(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        cache.set("key", "old")
        cache.set("key", "new")
        assert cache.get("key") == "new"

    def test_delete(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        cache.set("key", "value")
        cache.delete("key")
        assert cache.get("key") is None

    def test_ttl_expiry(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        cache.set("key", "value", ttl=1)
        assert cache.get("key") == "value"
        time.sleep(1.5)
        assert cache.get("key") is None

    def test_graceful_degradation(self):
        cache = RedisCache(enabled=True)
        cache.connect(host="127.0.0.1", port=16379)
        cache.set("key", "value")
        assert cache.get("key") == "value"
        cache.delete("key")
        assert cache.get("key") is None

    def test_disconnect_twice(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        assert cache.disconnect() is True
        assert cache.disconnect() is True

    def test_get_after_disconnect(self):
        cache = RedisCache(enabled=False)
        cache.connect()
        cache.set("k", "v")
        cache.disconnect()
        assert cache.get("k") is None