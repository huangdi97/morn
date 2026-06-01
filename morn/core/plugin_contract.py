"""YAML 插件契约解析器——解析 PluginContract dataclass"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Union

import yaml


@dataclass
class PluginContract:
    plugin_id: str
    name: str = ""
    version: str = "0.1.0"
    type: str = "plugin"
    dependencies: list = field(default_factory=list)
    required_permissions: list = field(default_factory=list)
    optional_permissions: list = field(default_factory=list)
    needs_periodic_trigger: bool = False
    usage_hint: str = "low"
    health_check_interval: int = 60


def parse_contract(yaml_path: Union[str, Path]) -> PluginContract:
    path = Path(yaml_path)
    if not path.exists():
        raise FileNotFoundError(f"Contract file not found: {yaml_path}")

    with open(path, "r") as f:
        data = yaml.safe_load(f)

    if not isinstance(data, dict):
        raise ValueError("Contract must be a YAML mapping")

    if "plugin_id" not in data:
        raise ValueError("Missing required field: plugin_id")

    return PluginContract(
        plugin_id=str(data["plugin_id"]),
        name=str(data.get("name", "")),
        version=str(data.get("version", "0.1.0")),
        type=str(data.get("type", "plugin")),
        dependencies=list(data.get("dependencies", [])),
        required_permissions=list(data.get("required_permissions", [])),
        optional_permissions=list(data.get("optional_permissions", [])),
        needs_periodic_trigger=bool(data.get("needs_periodic_trigger", False)),
        usage_hint=str(data.get("usage_hint", "low")),
        health_check_interval=int(data.get("health_check_interval", 60)),
    )
