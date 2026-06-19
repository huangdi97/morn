#!/usr/bin/env python3
"""Hello World plugin — minimal Morn external plugin example."""

import sys
import json


def handle_request(method: str, params: dict) -> dict:
    if method == "ping":
        return {"pong": True, "message": "Hello from Morn plugin!"}
    elif method == "greet":
        name = params.get("name", "World")
        return {"result": f"Hello, {name}!"}
    return {"error": f"Unknown method: {method}"}


for line in sys.stdin:
    try:
        req = json.loads(line.strip())
        response = handle_request(
            req.get("method", ""),
            req.get("params", {}),
        )
        response["id"] = req.get("id")
        print(json.dumps(response), flush=True)
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"Invalid JSON: {e}", "id": None}), flush=True)
    except Exception as e:
        print(json.dumps({"error": str(e), "id": None}), flush=True)
