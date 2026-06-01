"""Morn 内置插件"""
from .health_monitor import HealthMonitorPlugin
from .dream_engine import DreamEnginePlugin
from .self_reflection import SelfReflectionPlugin
from .identity_affirmer import IdentityAffirmerPlugin
from .self_pruner import SelfPrunerPlugin
from .bond_tracker import BondTrackerPlugin
from .intent_drift import IntentDriftPlugin
from .audit import AuditPlugin
from .thinking_evolution import ThinkingEvolutionPlugin
from .milestones import MilestonePlugin
from .hindsight import HindsightPlugin
from .example_hello import ExampleHelloPlugin

__all__ = [
    "HealthMonitorPlugin",
    "DreamEnginePlugin",
    "SelfReflectionPlugin",
    "IdentityAffirmerPlugin",
    "SelfPrunerPlugin",
    "BondTrackerPlugin",
    "IntentDriftPlugin",
    "AuditPlugin",
    "ThinkingEvolutionPlugin",
    "MilestonePlugin",
    "HindsightPlugin",
    "ExampleHelloPlugin",
]