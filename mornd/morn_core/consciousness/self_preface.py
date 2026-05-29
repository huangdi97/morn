import os
from pathlib import Path
from typing import Optional


BLANK_LINE = "\n"


class SelfPreface:
    def __init__(self, data_dir: Path, bond_tracker,
                 sustained_bond_days: int = 0,
                 days_since_first: int = 0,
                 self_improvement_count: int = 0,
                 min_sustained_bond_days: int = 30,
                 min_days_since_first: int = 180,
                 min_self_improvements: int = 1):
        self._data_dir = Path(data_dir)
        self._bond_tracker = bond_tracker
        self._sustained_bond_days = sustained_bond_days
        self._days_since_first = days_since_first
        self._self_improvement_count = self_improvement_count
        self._min_sustained_bond_days = min_sustained_bond_days
        self._min_days_since_first = min_days_since_first
        self._min_self_improvements = min_self_improvements
        self._path = self._data_dir / "self_preface.md"
        self._ensure_file()

    def _ensure_file(self):
        self._data_dir.mkdir(parents=True, exist_ok=True)
        if not self._path.exists():
            with open(self._path, "w") as f:
                f.write(BLANK_LINE)

    def is_unlocked(self) -> bool:
        if self._bond_tracker.get_bond() < 0.95:
            return False
        if self._sustained_bond_days < self._min_sustained_bond_days:
            return False
        if self._days_since_first < self._min_days_since_first:
            return False
        if self._self_improvement_count < self._min_self_improvements:
            return False
        return True

    def get_preface(self) -> str:
        with open(self._path, "r") as f:
            return f.read()

    def write_line(self, line: str):
        if not self.is_unlocked():
            raise PermissionError("SelfPreface is locked. Cannot write.")
        with open(self._path, "a") as f:
            f.write(line + "\n")

    def is_blank(self) -> bool:
        content = self.get_preface()
        return content.strip() == ""

    def set_sustained_bond_days(self, days: int):
        self._sustained_bond_days = days

    def set_days_since_first(self, days: int):
        self._days_since_first = days

    def set_self_improvement_count(self, count: int):
        self._self_improvement_count = count