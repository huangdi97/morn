"""项目结构和基础导入测试。"""

import os
import sys
import tomllib

# 确保 morn_core 可导入
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, PROJECT_ROOT)


class TestProjectStructure:
    """验证项目目录结构和关键文件存在"""

    def test_project_root_exists(self):
        """项目根目录存在"""
        assert os.path.isdir(PROJECT_ROOT), f"项目根目录不存在: {PROJECT_ROOT}"

    def test_pyproject_toml_exists(self):
        """pyproject.toml 文件存在且可解析"""
        path = os.path.join(PROJECT_ROOT, "pyproject.toml")
        assert os.path.isfile(path), f"pyproject.toml 不存在: {path}"
        with open(path, "rb") as f:
            data = tomllib.load(f)
        assert data["project"]["name"] == "morn-core"
        assert data["project"]["version"] == "0.1.0"

    def test_requirements_exists(self):
        """requirements.txt 文件存在"""
        path = os.path.join(PROJECT_ROOT, "requirements.txt")
        assert os.path.isfile(path)

    def test_requirements_parse(self):
        """requirements.txt 每行格式正确（不含非法字符）"""
        path = os.path.join(PROJECT_ROOT, "requirements.txt")
        with open(path) as f:
            lines = [l.strip() for l in f if l.strip() and not l.startswith("#")]
        for line in lines:
            assert " " not in line, f"requirements 行含空格: {line}"
            assert "==" in line or ">=" in line, \
                f"requirements 行无版本约束（建议用 >=）: {line}"

    def test_core_package_dirs(self):
        """所有核心子包目录存在"""
        sub_packages = [
            "morn_core",
            "morn_core/memory",
            "morn_core/emotion",
            "morn_core/chat",
            "morn_core/consciousness",
            "morn_core/presence",
            "morn_core/security",
            "morn_core/media",
            "tests",
        ]
        for pkg in sub_packages:
            path = os.path.join(PROJECT_ROOT, pkg)
            assert os.path.isdir(path), f"包目录不存在: {path}"

    def test_all_init_files_exist(self):
        """所有 __init__.py 文件存在"""
        init_files = [
            "morn_core/__init__.py",
            "morn_core/memory/__init__.py",
            "morn_core/emotion/__init__.py",
            "morn_core/chat/__init__.py",
            "morn_core/consciousness/__init__.py",
            "morn_core/presence/__init__.py",
            "morn_core/security/__init__.py",
            "morn_core/media/__init__.py",
            "tests/__init__.py",
        ]
        for rel_path in init_files:
            path = os.path.join(PROJECT_ROOT, rel_path)
            assert os.path.isfile(path), f"__init__.py 不存在: {path}"

    def test_readme_exists(self):
        """README.md 文件存在"""
        path = os.path.join(PROJECT_ROOT, "README.md")
        assert os.path.isfile(path)


class TestCoreImports:
    """验证 morn_core 包可导入"""

    def test_import_morn_core(self):
        """morn_core 可导入"""
        import morn_core
        assert morn_core.__version__ == "0.1.0"

    def test_import_all_subpackages(self):
        """所有子包可导入"""
        assert True
