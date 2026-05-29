"""A 级高级记忆组件"""
from .raw_snapshot_store import RawSnapshotStore
from .external_memory import ExternalMemoryAdapter
from .graph_store import GraphStore
from .knowledge_extractor import auto_extract, extract_knowledge
from .l4_depositor import check_and_deposit
