import json
import logging
import re
from pathlib import Path

logger = logging.getLogger("morn.evolution")


class SkillLoader:
    def __init__(self, skill_manager, evolution_logger=None):
        self._skill_manager = skill_manager
        self._evolution_logger = evolution_logger
        self._loaded_names = set()

    async def load_from_file(self, file_path):
        path = Path(file_path)
        if not path.exists():
            logger.warning("SKILL.md not found: %s", path)
            return False, "file_not_found"
        content = path.read_text(encoding="utf-8")
        parsed = self._parse_skill_md(content)
        if parsed is None:
            logger.warning("Invalid SKILL.md format: %s", path)
            return False, "invalid_format"
        name = parsed.get("name")
        if not name:
            logger.warning("SKILL.md missing name: %s", path)
            return False, "missing_name"
        if name in self._loaded_names:
            return False, "already_loaded"
        trigger_keywords = parsed.get("trigger_keywords")
        if not trigger_keywords:
            trigger_keywords = [path.stem]
        template = parsed.get("template", "")
        try:
            sid, is_new = await self._skill_manager.store.add_skill(
                name=name,
                trigger_keywords=trigger_keywords,
                template=template,
                source="external",
            )
            if not is_new:
                return False, "duplicate"
            self._loaded_names.add(name)
            if self._evolution_logger:
                self._evolution_logger.log("skill_loader", "load", {
                    "file": str(path),
                    "name": name,
                    "skill_id": sid,
                })
            return True, "loaded"
        except Exception as e:
            logger.warning("Failed to register skill %s: %s", name, e)
            return False, f"register_error:{e}"

    async def load_from_dir(self, dir_path):
        path = Path(dir_path)
        if not path.exists():
            return 0
        count = 0
        for md_file in sorted(path.glob("*.md")):
            ok, _ = await self.load_from_file(md_file)
            if ok:
                count += 1
        return count

    def list_available(self, dir_path):
        path = Path(dir_path)
        if not path.exists():
            return []
        results = []
        for md_file in sorted(path.glob("*.md")):
            results.append({
                "path": str(md_file),
                "name": md_file.stem,
            })
        return results

    def validate(self, file_path):
        path = Path(file_path)
        if not path.exists():
            return {"valid": False, "errors": ["file_not_found"]}
        content = path.read_text(encoding="utf-8")
        parsed = SkillLoader._parse_skill_md(content)
        if parsed is None:
            return {"valid": False, "errors": ["invalid_frontmatter"]}
        errors = []
        if not parsed.get("name"):
            errors.append("missing_name")
        return {"valid": len(errors) == 0, "errors": errors}

    @staticmethod
    def _parse_skill_md(content):
        m = re.match(r'^---\s*\n(.*?)\n---(?:[\s\n]*(.*))?$', content, re.DOTALL)
        if not m:
            return None
        frontmatter_text = m.group(1)
        body = (m.group(2) or "").strip()
        frontmatter = {}
        current_key = None
        for line in frontmatter_text.strip().split("\n"):
            stripped = line.strip()
            if not stripped:
                continue
            if ":" in stripped and not stripped.startswith("- "):
                key, _, value = stripped.partition(":")
                current_key = key.strip()
                value = value.strip()
                if value:
                    if value.startswith("[") and value.endswith("]"):
                        try:
                            value = json.loads(value)
                        except (json.JSONDecodeError, TypeError):
                            value = [v.strip().strip('"').strip("'") for v in value[1:-1].split(",") if v.strip()]
                    frontmatter[current_key] = value
                else:
                    frontmatter[current_key] = []
            elif stripped.startswith("- ") and current_key:
                item = stripped[2:].strip().strip('"').strip("'")
                if not isinstance(frontmatter.get(current_key), list):
                    frontmatter[current_key] = []
                frontmatter[current_key].append(item)
        name = frontmatter.get("name", "")
        if isinstance(name, str):
            name = name.strip().strip('"').strip("'")
        description = frontmatter.get("description", "")
        if isinstance(description, str):
            description = description.strip().strip('"').strip("'")
        trigger_keywords = frontmatter.get("trigger_keywords", [])
        if isinstance(trigger_keywords, str):
            trigger_keywords = [trigger_keywords]
        template = frontmatter.get("template", body)
        if isinstance(template, str):
            template = template.strip()
        return {
            "name": name or None,
            "description": description or None,
            "trigger_keywords": trigger_keywords,
            "template": template,
        }