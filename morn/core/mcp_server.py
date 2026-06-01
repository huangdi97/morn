"""MCP Server — 自动将插件注册为 MCP 工具（stdio 模式 JSON-RPC 2.0）"""

import asyncio
import json
import logging
import sys
from typing import Optional

logger = logging.getLogger("morn.mcp")


class MCPServer:
    """MCP Server — stdio 模式 JSON-RPC 2.0，自动将插件 capabilities 注册为 MCP tools。

    约定：stdout 专用于 MCP JSON-RPC 通信，Morn 自身输出（日志/CLI）应定向到 stderr。
    MCPServer 日志通过 logging 模块自动输出到 stderr。
    """
    def __init__(self, enabled: bool = True):
        self._enabled = enabled
        self._plugins: dict[str, dict] = {}
        self._server_task: Optional[asyncio.Task] = None
        self._reader: Optional[asyncio.StreamReader] = None
        self._writer: Optional[asyncio.StreamWriter] = None

    @property
    def enabled(self) -> bool:
        return self._enabled

    def register_plugin(self, plugin) -> None:
        if not self._enabled:
            return
        plugin_id = plugin.plugin_id
        caps = list(getattr(plugin, "capabilities", []))
        self._plugins[plugin_id] = {
            "plugin": plugin,
            "capabilities": caps,
        }
        logger.info("registered plugin %s with %d capabilities", plugin_id, len(caps))

    def unregister_plugin(self, plugin_id: str) -> None:
        self._plugins.pop(plugin_id, None)
        logger.info("unregistered plugin %s", plugin_id)

    def get_tools(self) -> list[dict]:
        tools = []
        for plugin_id, info in self._plugins.items():
            for cap in info["capabilities"]:
                tool = {
                    "name": f"{plugin_id}__{cap.get('name', 'unknown')}",
                    "description": cap.get("description", ""),
                    "inputSchema": cap.get(
                        "parameters",
                        {"type": "object", "properties": {}},
                    ),
                }
                tools.append(tool)
        return tools

    async def start(self):
        if not self._enabled:
            return
        if self._server_task is not None:
            return
        self._server_task = asyncio.create_task(self._stdio_loop())
        logger.info("MCP server started (stdio mode)")

    async def stop(self):
        if self._server_task:
            self._server_task.cancel()
            try:
                await self._server_task
            except asyncio.CancelledError:
                pass
            self._server_task = None
            logger.info("MCP server stopped")

    async def _stdio_loop(self):
        loop = asyncio.get_event_loop()
        try:
            self._reader = asyncio.StreamReader()
            protocol = asyncio.StreamReaderProtocol(self._reader)
            await loop.connect_read_pipe(lambda: protocol, sys.stdin)
        except (PermissionError, OSError, NotImplementedError):
            logger.warning("stdin not available for MCP — running in tool-list-only mode")
            return

        async def write_response(data: dict):
            line = json.dumps(data, ensure_ascii=False) + "\n"
            sys.stdout.write(line)
            sys.stdout.flush()

        while True:
            try:
                raw = await self._reader.readline()
            except asyncio.CancelledError:
                break
            if not raw:
                break
            raw = raw.strip()
            if not raw:
                continue
            try:
                msg = json.loads(raw)
            except json.JSONDecodeError:
                continue

            method = msg.get("method")
            msg_id = msg.get("id")
            params = msg.get("params", {})

            if method == "initialize":
                await write_response({
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {"tools": {}},
                        "serverInfo": {
                            "name": "morn-mcp",
                            "version": "0.1.0",
                        },
                    },
                })
            elif method == "notifications/initialized":
                pass
            elif method == "tools/list":
                await write_response({
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "result": {"tools": self.get_tools()},
                })
            elif method == "tools/call":
                tool_name = params.get("name", "")
                tool_args = params.get("arguments", {})
                result = await self._handle_tool_call(tool_name, tool_args)
                await write_response({
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "result": result,
                })
            else:
                if msg_id is not None:
                    await write_response({
                        "jsonrpc": "2.0",
                        "id": msg_id,
                        "error": {"code": -32601, "message": f"Method not found: {method}"},
                    })

    async def _handle_tool_call(self, tool_name: str, args: dict) -> dict:
        if "__" not in tool_name:
            return {"content": [{"type": "text", "text": f"Unknown tool: {tool_name}"}]}
        plugin_id, _, cap_name = tool_name.partition("__")
        info = self._plugins.get(plugin_id)
        if info is None:
            return {"content": [{"type": "text", "text": f"Plugin not found: {plugin_id}"}]}
        plugin = info["plugin"]
        handler_name = f"mcp_{cap_name}"
        handler = getattr(plugin, handler_name, None)
        if handler is None:
            attr = getattr(plugin, cap_name, None)
            if attr is None:
                return {"content": [{"type": "text", "text": f"Capability not found: {cap_name}"}]}
            if asyncio.iscoroutinefunction(attr):
                result = await attr(**args)
            else:
                result = attr(**args)
        else:
            result = await handler(args)
        if not isinstance(result, dict):
            result = {"result": result}
        return {"content": [{"type": "text", "text": json.dumps(result, ensure_ascii=False)}]}
