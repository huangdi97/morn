import logging
import time
from typing import Optional

logger = logging.getLogger("morn.memory")


class RedisCache:
    def __init__(self, host="localhost", port=6379, db=0, enabled=False):
        self._host = host
        self._port = port
        self._db = db
        self._enabled = enabled
        self._client = None
        self._available = False
        self._local: dict[str, tuple[str, float]] = {}

    def connect(self, host="localhost", port=6379, db=0):
        self._host = host
        self._port = port
        self._db = db
        if not self._enabled:
            self._available = False
            return True
        try:
            import redis
            self._client = redis.Redis(host=host, port=port, db=db,
                                       socket_connect_timeout=1, socket_timeout=1)
            self._client.ping()
            self._available = True
        except Exception:
            self._available = False
            logger.warning("Redis unavailable, falling back to local memory")
        return True

    def disconnect(self):
        if self._client:
            try:
                self._client.close()
            except Exception:
                pass
        self._client = None
        self._available = False
        self._local.clear()
        return True

    def is_connected(self):
        if not self._enabled or not self._client:
            return False
        try:
            return self._client.ping()
        except Exception:
            self._available = False
            return False

    def set(self, key, value, ttl=None):
        if self._available and self._client:
            try:
                self._client.set(key, value, ex=ttl)
                return
            except Exception:
                self._available = False
                logger.warning("Redis set failed, falling back to local memory")
        self._local[key] = (value, time.time() + ttl if ttl else 0.0)

    def get(self, key):
        if self._available and self._client:
            try:
                val = self._client.get(key)
                if val is not None:
                    return val.decode("utf-8") if isinstance(val, bytes) else val
                return None
            except Exception:
                self._available = False
                logger.warning("Redis get failed, falling back to local memory")
        entry = self._local.get(key)
        if entry is None:
            return None
        value, expiry = entry
        if expiry and time.time() > expiry:
            del self._local[key]
            return None
        return value

    def delete(self, key):
        if self._available and self._client:
            try:
                self._client.delete(key)
                return
            except Exception:
                self._available = False
                logger.warning("Redis delete failed, falling back to local memory")
        self._local.pop(key, None)