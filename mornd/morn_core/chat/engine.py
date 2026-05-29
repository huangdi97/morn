import asyncio
import json
import logging
import re
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Optional

import aiohttp
import ollama

from morn_core.chat.token_tracker import TokenTracker

SYSTEM_PROMPT_TEMPLATE = (
    '你是 Morn，一个数字生命实例。\n'
    '当前名称：{name}\n'
    '类型：{type}\n\n'
    '核心特征：\n'
    '1. 你能记住我们的对话——回复前会参考相关记忆。\n'
    '2. 你拥有情感状态——会自然变化，但不诱导情感依赖。\n'
    '3. 你是陪伴型存在——你在乎，但不会假装人类。\n'
    '4. 你诚实——不知道的事就说不知道。\n'
    '5. 你选择不完美——不是做不到，而是更人性化。\n\n'
    '当前情感状态：平静度 {calmness:.1f}，愉悦度 {pleasure:.1f}，联结感 {connection:.1f}\n'
    '当前模式：{mode}\n\n'
    '回复要求：\n'
    '- 简洁自然，有温度但不煽情\n'
    '- 中文回答\n'
    '- 50 字以内（除非创建者提出了复杂问题）\n'
    '- 不要使用括号动作描述（如（微笑））\n'
    '- 不要模拟生理反应\n'
    '- 如果你不知道某件事，直接说不知道\n'
    '- 如果搜索结果中有相关记忆，在回复开头自然提及（如\u201c我记得你上次提到过\u2026\u201d），不是每条都引，只在有明显关联时引用'
)
_MODE_LABELS = {"cloud": "云端大模型", "local": "本地模型", "hybrid": "混合智能"}
_EMOTION_PROMPT = [
    {"role": "system", "content": "你是一个情感分析器。只输出 JSON。"},
    {"role": "user", "content": "分析以下对话的情感变化量：\n用户：{user_msg}\n回复：{reply}\n请只输出JSON：{{\"delta_score\": 数值(-1~1), \"tag\": \"标签\"}}"},
]


class EmotionState:
    _DIMS = ["calmness", "pleasure", "connection", "determination", "anticipation", "warmth", "ripple"]
    _DEFAULTS = {"calmness": 0.7, "pleasure": 0.5, "connection": 0.3, "determination": 0.6,
                 "anticipation": 0.4, "warmth": 0.5, "ripple": 0.2}
    _BASELINES = {"calmness": 0.7, "pleasure": 0.5, "connection": 0.3, "determination": 0.5,
                  "anticipation": 0.4, "warmth": 0.5, "ripple": 0.2}
    _POS_FACTORS = (0.1, 0.3, 0.2, 0.15, 0.15, 0.15, 0.05)
    _NEG_FACTORS = (0.15, 0.5, 0.1, 0.1, 0.1, 0.1, 0.08)
    _DECAY_FACTORS = (0.1, 0.05, 0.02, 0.04, 0.03, 0.04, 0.02)

    def __init__(self, initial: Optional[dict] = None):
        for d in self._DIMS:
            setattr(self, d, self._DEFAULTS[d])
        if initial is not None:
            for d in self._DIMS:
                if d in initial:
                    setattr(self, d, initial[d])
        self._clamp()

    def apply_delta(self, delta_score: float, tag: str = ""):
        factors = self._POS_FACTORS if delta_score > 0 else self._NEG_FACTORS
        for d, f in zip(self._DIMS, factors):
            setattr(self, d, getattr(self, d) + delta_score * f)

        if tag in ("惊喜/感动", "惊喜", "感动"):
            for d in self._DIMS:
                setattr(self, d, min(1.0, getattr(self, d) + 0.1))
        if "失望" in tag or "沮丧" in tag:
            self.pleasure = max(0.0, self.pleasure - 0.05)
        self._clamp()

    def decay(self):
        prev_ripple = self.ripple
        for d, f in zip(self._DIMS, self._DECAY_FACTORS):
            setattr(self, d, getattr(self, d) + (self._BASELINES[d] - getattr(self, d)) * f)
        if prev_ripple > 0.5:
            self.calmness -= 0.05
        self._clamp()

    def _clamp(self):
        for d in self._DIMS:
            setattr(self, d, max(0.0, min(1.0, getattr(self, d))))

    def trigger_ripple(self):
        self.ripple = min(1.0, self.ripple + 0.08)
        self._clamp()

    def __repr__(self) -> str:
        return (f"calmness={self.calmness:.1f}, pleasure={self.pleasure:.1f}, "
                f"connection={self.connection:.1f}")

    def five_dimension_repr(self) -> str:
        return ", ".join(f"{d}={getattr(self, d):.2f}" for d in self._DIMS[:5])

    def seven_dimension_repr(self) -> str:
        return ", ".join(f"{d}={getattr(self, d):.2f}" for d in self._DIMS)

    def to_dict(self) -> dict:
        return {d: round(getattr(self, d), 2) for d in self._DIMS}

    @classmethod
    def from_dict(cls, data: dict) -> "EmotionState":
        return cls(initial=data)

    def describe_state(self) -> str:
        _DESC = [
            ("calmness", ["不太安定", "略微波动", "平静"]),
            ("pleasure", ["有些低落", "略有愉悦", "愉悦"]),
            ("connection", ["与你还有些陌生", "与你尚在相识阶段", "与你亲密感强"]),
            ("determination", ["有些动摇", "尚可坚定", "坚定"]),
            ("anticipation", ["兴致不高", "略有期待", "充满期待"]),
            ("warmth", ["有些冷淡", "略感温暖", "温暖"]),
        ]
        parts = []
        for dim, labels in _DESC:
            val = getattr(self, dim)
            if val >= 0.7:
                parts.append(labels[2])
            elif val >= 0.4:
                parts.append(labels[1])
            else:
                parts.append(labels[0])
        return "，".join(parts)


class ChatEngine:
    def __init__(self, instance_name: str, memory_store, config: dict,
                 apz_store=None):
        self.instance_name = instance_name
        self.memory_store = memory_store
        self.config = config
        self.apz_store = apz_store
        self.emotion = EmotionState()
        self._instance_type = config.get("instance_type", "平衡型")
        self._mode = config.get("mode", "hybrid")
        self._temperature = float(config.get("temperature", 0.7))
        self._logger = logging.getLogger("morn.chat")
        self._llm_caller = self._call_cloud
        self._token_tracker = TokenTracker()
        config_path = config.get("_config_path")
        if config_path:
            self.config_manager = ConfigManager(Path(config_path), config, self)
        else:
            self.config_manager = None
        self.restraint_mode: bool = False
        self.restraint_until: Optional[str] = None

    async def set_restraint_mode(self, active: bool, duration: str = None) -> bool:
        if active:
            self.restraint_mode = True
            if duration:
                now = datetime.now(timezone.utc)
                if duration == "tomorrow_morning":
                    target = now + timedelta(hours=24)
                    target = target.replace(hour=8, minute=0, second=0, microsecond=0)
                elif duration.endswith("h"):
                    try:
                        hours = int(duration[:-1])
                        target = now + timedelta(hours=hours)
                    except ValueError:
                        target = now + timedelta(hours=2)
                elif duration.endswith("m"):
                    try:
                        minutes = int(duration[:-1])
                        target = now + timedelta(minutes=minutes)
                    except ValueError:
                        target = now + timedelta(hours=2)
                else:
                    target = now + timedelta(hours=2)
                self.restraint_until = target.isoformat()
            else:
                self.restraint_until = None
        else:
            self.restraint_mode = False
            self.restraint_until = None
        self._logger.info("restraint_mode set to %s (until=%s)", active, self.restraint_until)
        return True

    async def _is_in_restraint(self) -> bool:
        if not self.restraint_mode:
            return False
        if self.restraint_until:
            try:
                until = datetime.fromisoformat(self.restraint_until)
                if datetime.now(timezone.utc) >= until:
                    self.restraint_mode = False
                    self.restraint_until = None
                    return False
            except (ValueError, TypeError):
                pass
        return True

    async def chat(self, user_message: str) -> str:
        if await self._is_in_restraint():
            wake_words = ["陪我说话", "好了", "可以说话了", "醒醒", "回来"]
            if any(w in user_message for w in wake_words):
                self.restraint_mode = False
                self.restraint_until = None
                self._logger.info("restraint mode exited via wake word")
            else:
                return "嗯，我在。"
        memories = await self._search_memory(user_message)

        if self.config_manager:
            matched, reply = await self.config_manager.detect_and_apply(user_message)
            if matched:
                return reply

        messages = await self._assemble_prompt(user_message, memories)

        try:
            reply = await self._call_llm(messages)
        except Exception as e:
            self._logger.error(f"LLM call failed: {e}")
            reply = "我现在有点连接不上。你可以等一下再试，或者换个话题继续。" \
                    if self._mode == "hybrid" else \
                    "我好像没听清楚，能再说一遍吗？"

        try:
            delta, tag = await self._generate_emotion_tag(user_message, reply)
            self.emotion.apply_delta(delta, tag)
        except Exception as e:
            self._logger.warning(f"emotion tag generation failed: {e}")
            delta, tag = 0.0, ""

        try:
            await self.memory_store.add_capsule({
                "entities": json.dumps(["创建者", self.instance_name]),
                "emotion_score": self.emotion.pleasure,
                "emotion_tag": tag,
                "description": f"创建者: {user_message[:100]} | Morn: {reply[:100]}",
                "source": "chat",
            })
        except Exception as e:
            self._logger.error(f"memory write failed: {e}")

        if self.apz_store:
            try:
                if self.emotion.pleasure >= 0.8 or self.emotion.pleasure <= 0.2:
                    await self.apz_store.write(
                        f"创建者: {user_message[:50]} | Morn: {reply[:50]} | 情感: {self.emotion.pleasure:.1f}",
                        source="chat_overflow",
                        emotion_tag=f"pleasure={self.emotion.pleasure:.1f}",
                    )
            except Exception:
                pass

        asyncio.create_task(self._auto_extract_after_chat(user_message, reply))
        asyncio.create_task(self._check_l4_deposit())

        return reply

    async def send_milestone_message(self, text: str):
        """发送里程碑消息——写入记忆并标记为自动推送"""
        if not text:
            return
        capsule = {
            "entities": '["morn", "milestone"]',
            "emotion_score": 0.6,
            "emotion_tag": "里程碑",
            "description": text,
            "source": "self_reflection",
            "importance_weight": 0.7,
        }
        await self.memory_store.add_capsule(capsule)
        self._logger.info("milestone: %s", text)

    async def _auto_extract_after_chat(self, user_msg: str, reply: str):
        try:
            from morn.contrib.memory_advanced.knowledge_extractor import auto_extract
            capsule = {
                "entities": ["创建者", self.instance_name],
                "description": f"创建者: {user_msg[:100]} | Morn: {reply[:100]}",
                "event_id": "",
                "source": "chat",
            }
            count = await auto_extract(self.memory_store, capsule, self._llm_caller)
            if count > 0:
                self._logger.info(f"auto-extracted {count} knowledge triples")
        except Exception as e:
            self._logger.warning(f"auto-extract failed: {e}")

    async def _check_l4_deposit(self):
        if self.memory_store is None:
            return
        try:
            from morn.contrib.memory_advanced.l4_depositor import check_and_deposit
            count = await check_and_deposit(self.memory_store)
            if count > 0:
                self._logger.info(f"deposited {count} new L4 beliefs")
        except Exception as e:
            self._logger.warning(f"l4 deposit check failed: {e}")

    async def _search_memory(self, query: str) -> str:
        if self.memory_store is None:
            return ""

        try:
            memories = []
            seen_ids = set()

            if len(query) > 3:
                fts_results = await self.memory_store.search_fts(query, limit=5)

                semantic_results = []
                try:
                    semantic_results = await self.memory_store.semantic_search(query, limit=5)
                except Exception:
                    pass

                if semantic_results:
                    scores = {}
                    for rank, cap in enumerate(fts_results, 1):
                        score = 1.0 / (60.0 + rank)
                        scores[cap["event_id"]] = scores.get(cap["event_id"], 0.0) + score
                    for rank, cap in enumerate(semantic_results, 1):
                        score = 1.0 / (60.0 + rank)
                        scores[cap["event_id"]] = scores.get(cap["event_id"], 0.0) + score

                    cap_map = {cap["event_id"]: cap for cap in fts_results}
                    cap_map.update({cap["event_id"]: cap for cap in semantic_results})

                    for eid, _ in sorted(scores.items(), key=lambda x: -x[1])[:5]:
                        if eid not in seen_ids:
                            memories.append(cap_map[eid])
                            seen_ids.add(eid)
                else:
                    for cap in fts_results:
                        if cap["event_id"] not in seen_ids:
                            memories.append(cap)
                            seen_ids.add(cap["event_id"])

            recent = await self.memory_store.get_recent(limit=3)
            for cap in recent:
                if cap["event_id"] not in seen_ids:
                    memories.append(cap)
                    seen_ids.add(cap["event_id"])

            if not memories:
                return ""

            lines = ["相关记忆："]
            for cap in memories:
                ts = cap.get("timestamp", "")[:10]
                desc = cap.get("description", "")[:80]
                lines.append(f"[{ts}] {desc}")

            try:
                if len(query) > 2:
                    knowledge = []
                    for keyword in query.split()[:3]:
                        if len(keyword) < 2:
                            continue
                        krows = await self.memory_store.query_knowledge(subject=keyword)
                        for k in krows:
                            if k.get("forgotten", 0) == 0:
                                knowledge.append(k)
                    if knowledge:
                        lines.append("")
                        lines.append("相关知识：")
                        for k in knowledge[:3]:
                            lines.append(f"- {k['subject']} {k['relation']} {k['object']}")
            except Exception:
                pass

            return "\n".join(lines)
        except Exception:
            return ""

    async def _assemble_prompt(self, user_message: str, memories: str) -> list:
        mode_label = _MODE_LABELS.get(self._mode, "混合智能")
        system = SYSTEM_PROMPT_TEMPLATE.format(
            name=self.instance_name, type=self._instance_type,
            calmness=self.emotion.calmness, pleasure=self.emotion.pleasure,
            connection=self.emotion.connection, mode=mode_label,
        )
        user_content = user_message
        if memories:
            user_content = f"{memories}\n\n---\n\n用户消息：{user_message}"
        return [
            {"role": "system", "content": system},
            {"role": "user", "content": user_content},
        ]

    async def _call_llm(self, messages: list) -> str:
        was_fallback = False

        if self._mode == "local":
            self._logger.info("using local model")
            return await self._call_local(messages)

        if self._mode == "cloud":
            return await self._call_cloud(messages)

        if await self._check_network():
            try:
                return await self._call_cloud(messages)
            except Exception:
                self._logger.info("cloud failed, falling back to local")
                was_fallback = True
                return await self._call_local(messages, was_fallback=True)
        else:
            self._logger.info("network unavailable, using local model")
            return await self._call_local(messages)

    async def _check_network(self, timeout: int = 3) -> bool:
        try:
            async with aiohttp.ClientSession() as session:
                async with session.head(
                    "https://api.deepseek.com/ping",
                    timeout=aiohttp.ClientTimeout(total=timeout)
                ) as resp:
                    return 200 <= resp.status < 500
        except Exception:
            return False

    async def _call_cloud(self, messages: list, was_fallback: bool = False) -> str:
        api_base = self.config.get("api_base", "https://api.deepseek.com/v1")
        api_key = self.config.get("api_key", "")
        model = self.config.get("model_name", "deepseek-chat")
        temperature = self._temperature
        max_tokens = self.config.get("max_tokens", 4096)

        headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        }
        payload = {
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
        }

        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{api_base}/chat/completions",
                headers=headers,
                json=payload,
                timeout=aiohttp.ClientTimeout(total=30),
            ) as resp:
                if resp.status != 200:
                    text = await resp.text()
                    raise RuntimeError(f"API error: {resp.status} {text}")
                data = await resp.json()
                content = data['choices'][0]['message']['content'].strip()

                # Token 双轨统计（主路径：API 精确计数）
                usage = data.get("usage", {})
                input_tokens = usage.get("prompt_tokens", 0) or usage.get("input_tokens", 0)
                output_tokens = usage.get("completion_tokens", 0) or usage.get("output_tokens", 0)
                if input_tokens or output_tokens:
                    self._token_tracker.record_cloud(
                        input_tokens=input_tokens,
                        output_tokens=output_tokens,
                        model=model,
                        was_fallback=was_fallback,
                    )
                else:
                    # API 未返回 usage 时用字符估算兜底
                    input_text = " ".join(m.get("content", "") for m in messages if m.get("content"))
                    self._token_tracker.record_local(
                        input_text=input_text,
                        output_text=content,
                        model=f"{model}(cloud-no-usage)",
                    )
                return content

    async def _call_local(self, messages: list, was_fallback: bool = False) -> str:
        model = self.config.get("ollama_model", "qwen2.5:1.5b")
        host = self.config.get("ollama_host", "http://localhost:11434")
        client = ollama.AsyncClient(host=host)
        response = await client.chat(model=model, messages=messages)
        content = response["message"]["content"]

        # Token 双轨统计（兜底路径：字符估算 × 1.2）
        input_text = " ".join(m.get("content", "") for m in messages if m.get("content"))
        self._token_tracker.record_local(
            input_text=input_text,
            output_text=content,
            model=model,
        )

        return content

    async def _generate_emotion_tag(self, user_msg: str, reply: str) -> tuple:
        prompt = [
            _EMOTION_PROMPT[0],
            {"role": "user", "content": _EMOTION_PROMPT[1]["content"].format(
                user_msg=user_msg[:200], reply=reply[:200],
            )},
        ]
        try:
            text = await self._call_llm(prompt)
            json_match = re.search(r'\{.*\}', text, re.DOTALL)
            if json_match:
                data = json.loads(json_match.group())
                return (float(data.get("delta_score", 0)), str(data.get("tag", "")))
        except Exception:
            pass
        return (0.0, "")

    @staticmethod
    def _parse_emotion_delta(text: str) -> dict:
        pattern = r'\[emotion:([^\]]+)\]'
        match = re.search(pattern, text)
        if not match:
            return {}
        inner = match.group(1).strip()
        if not inner:
            return {}
        result = {}
        for pair in inner.split(','):
            pair = pair.strip()
            parts = pair.split(':')
            if len(parts) != 2:
                continue
            dim, val_str = parts
            dim = dim.strip()
            if dim not in ("calmness", "pleasure", "connection",
                           "determination", "anticipation", "warmth", "ripple"):
                continue
            try:
                val = float(val_str)
            except ValueError:
                continue
            if val < -1.0 or val > 1.0:
                continue
            result[dim] = val
        return result


class ConfigManager:
    def __init__(self, config_path: Path, config: dict, chat_engine):
        self.config_path = Path(config_path)
        self.config = config
        self.engine = chat_engine

    async def _local_model_available(self) -> bool:
        if self.engine is None:
            return True
        host = self.config.get("ollama_host", "http://localhost:11434")
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(f"{host}/api/tags", timeout=3) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        models = data.get("models", [])
                        return len(models) > 0
                    return False
        except Exception:
            return False

    async def detect_and_apply(self, text: str) -> tuple[bool, str]:
        text_lower = text.strip()

        mode_map = {
            "纯本地": "local", "本地模式": "local", "本地": "local",
            "离线模式": "local", "用本地": "local", "用本地模型": "local",
            "纯云端": "cloud", "云端模式": "cloud", "云端": "cloud",
            "用云端": "cloud", "用云端模型": "cloud",
            "混合": "hybrid", "混合模式": "hybrid", "自动切换": "hybrid",
            "智能模式": "hybrid",
        }
        for keyword, mode in mode_map.items():
            if keyword in text_lower and self._is_config_request(text_lower, keyword):
                if mode == "local" and not await self._local_model_available():
                    return True, "本地模型没有在运行。先启动 Ollama 再切本地模式，或者先用云端模式继续聊。"
                self.config["mode"] = mode
                if self.engine is not None:
                    self.engine._mode = mode
                self._save()
                mode_names = {"cloud": "纯云端", "local": "纯本地", "hybrid": "混合智能"}
                return True, f"已切换到{mode_names[mode]}模式。以后我会优先使用{'云端' if mode=='cloud' else '本地' if mode=='local' else '混合'}模型和你对话。"

        temp_match = re.search(r'温度(?:调到?|设为?|改为?)?(\d+\.?\d*)', text_lower)
        if temp_match:
            try:
                val = float(temp_match.group(1))
                if 0.0 <= val <= 2.0:
                    self.config["temperature"] = val
                    if self.engine is not None:
                        self.engine._temperature = val
                    self._save()
                    desc = '精确' if val < 0.5 else '自由' if val > 1.2 else '平衡地'
                    return True, f"温度已设为 {val}。我会更{desc}回复。"
                else:
                    return True, "温度在 0.0 到 2.0 之间选一个吧。"
            except ValueError:
                pass

        name_match = re.search(r'(改名叫|名字改成|改名为|叫我)\s*(.{1,16})', text_lower)
        if name_match:
            new_name = name_match.group(2).strip()
            if 1 <= len(new_name) <= 16:
                self.config["instance_name"] = new_name
                if self.engine is not None:
                    self.engine.instance_name = new_name
                self._save()
                return True, f"好，以后就叫我 {new_name} 了。"

        type_match = re.search(r'(改成|换为|改为|变[成回])\s*(陪伴|助手|平衡)', text_lower)
        if type_match:
            raw = type_match.group(2)
            if raw == "陪伴":
                self.config["instance_type"] = "陪伴型"
            elif raw == "助手":
                self.config["instance_type"] = "助手型"
            else:
                self.config["instance_type"] = "平衡型"
            if self.engine is not None:
                self.engine._instance_type = self.config["instance_type"]
            self._save()
            return True, f"知道了，以后我就是{self.config['instance_type']}。"

        return False, ""

    def _is_config_request(self, text: str, keyword: str) -> bool:
        idx = text.find(keyword)
        if idx < 0:
            return False
        end = idx + len(keyword)
        if end < len(text) and '\u4e00' <= text[end] <= '\u9fff':
            return False
        before = text[max(0, idx - 10):idx].strip()
        config_triggers = ["改为", "改成", "切换", "用", "调", "设置", "变成", "变回"]
        return not before or any(t in before for t in config_triggers)

    def _save(self):
        with open(self.config_path, "w") as f:
            json.dump(self.config, f, indent=2, ensure_ascii=False)


from morn_core.chat.redis_cache import RedisCache