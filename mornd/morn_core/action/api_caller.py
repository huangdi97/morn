import ipaddress
import logging
import socket
import time
from urllib.parse import urlparse

import requests


_PRIVATE_NETWORKS = [
    "127.0.0.0/8",
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
]


class APICaller:
    def __init__(self, config=None):
        self._config = config or {}
        self._allow_http = self._config.get("allow_http", False)
        self._logger = logging.getLogger("morn.api")
        self._history = []
        self._available = None

    def is_available(self):
        if self._available is not None:
            return self._available
        try:
            requests.get("https://1.1.1.1", timeout=3)
            self._available = True
        except Exception:
            self._available = False
        return self._available

    def validate(self, url):
        if not url or not url.strip():
            return False, "URL 为空"

        parsed = urlparse(url)
        if not parsed.scheme or not parsed.netloc:
            return False, "URL 格式无效"

        if parsed.scheme == "http" and not self._allow_http:
            return False, "非 HTTPS 请求被拦截（可通过 allow_http 配置放行）"

        hostname = parsed.hostname
        if not hostname:
            return False, "无法解析主机名"

        try:
            ip = socket.gethostbyname(hostname)
        except socket.gaierror:
            return False, f"无法解析域名: {hostname}"

        try:
            addr = ipaddress.ip_address(ip)
            for net in _PRIVATE_NETWORKS:
                if addr in ipaddress.ip_network(net, strict=False):
                    return False, f"内网请求被拦截: {ip} ({net})"
        except ValueError:
            pass

        return True, ""

    def call(self, method, url, headers=None, body=None, timeout=10):
        valid, reason = self.validate(url)
        if not valid:
            self._logger.warning("API 请求被拦截: %s %s (%s)", method, url, reason)
            return {"success": False, "error": reason, "status_code": 0}

        start = time.time()
        try:
            response = requests.request(
                method=method.upper(),
                url=url,
                headers=headers or {},
                json=body,
                timeout=timeout,
            )
            elapsed = time.time() - start
            entry = {
                "method": method.upper(),
                "url": url,
                "success": response.ok,
                "status_code": response.status_code,
                "body": response.text,
                "elapsed": elapsed,
            }
            self._history.append(entry)
            self._logger.info(
                "API 请求完成: %s %s (%d, %.2fs)",
                method.upper(), url, response.status_code, elapsed,
            )
            return entry
        except requests.Timeout:
            elapsed = time.time() - start
            entry = {
                "method": method.upper(),
                "url": url,
                "success": False,
                "error": f"请求超时 ({timeout}s)",
                "status_code": 0,
                "elapsed": elapsed,
            }
            self._history.append(entry)
            self._logger.warning("API 请求超时: %s %s (%ds)", method.upper(), url, timeout)
            return entry
        except Exception as e:
            elapsed = time.time() - start
            entry = {
                "method": method.upper(),
                "url": url,
                "success": False,
                "error": str(e),
                "status_code": 0,
                "elapsed": elapsed,
            }
            self._history.append(entry)
            self._logger.error("API 请求异常: %s %s (%s)", method.upper(), url, e)
            return entry

    def get_history(self):
        return list(self._history)
