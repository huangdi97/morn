import logging
import time

from morn_core.eventbus.bus import EventBus, Event, Priority
from morn_core.eventbus.hooks import HookManager, HookRegistration


def register_all_plugin_hooks(event_bus: EventBus, hook_manager: HookManager, state) -> None:
    register_dream_engine_hooks(event_bus, hook_manager, state)
    register_identity_hooks(event_bus, hook_manager, state)
    register_self_pruning_hooks(event_bus, hook_manager, state)
    register_bond_update_hooks(event_bus, hook_manager, state)
    register_intent_drift_hooks(event_bus, hook_manager, state)
    register_audit_hooks(event_bus, hook_manager, state)
    register_thinking_evolution_hooks(event_bus, hook_manager, state)
    register_milestone_hooks(event_bus, hook_manager, state)
    register_hindsight_hooks(event_bus, hook_manager, state)


def register_dream_engine_hooks(event_bus, hook_manager, state):
    async def on_minute(event):
        if not state.dream_engine:
            return
        idle = time.time() - state.last_interaction_time
        try:
            await state.dream_engine.tick(idle)
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "dream_engine", "error": str(e)},
                source="dream_engine", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="dream_engine", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_identity_hooks(event_bus, hook_manager, state):
    async def on_minute(event):
        if not state.identity_affirmer:
            return
        try:
            await state.identity_affirmer.tick()
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "identity_affirmer", "error": str(e)},
                source="identity_affirmer", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="identity_affirmer", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_self_pruning_hooks(event_bus, hook_manager, state):
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 10:
            return
        counter = 0
        if not state.self_pruner:
            return
        try:
            result = await state.self_pruner.diagnose()
            if result.get("capsules_pruned", 0) or result.get("skills_pruned", 0) or result.get("emotion_pruned", 0):
                await event_bus.publish(Event(
                    type="self_pruning.completed",
                    payload=result,
                    source="self_pruner", priority=Priority.LOW,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "self_pruner", "error": str(e)},
                source="self_pruner", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="self_pruner", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_bond_update_hooks(event_bus, hook_manager, state):
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 5:
            return
        counter = 0
        if not state.bond_tracker or not state.chat_engine:
            return
        try:
            idle = time.time() - state.last_interaction_time
            depth = min(state.heartbeat_count / 100, 1.0)
            sentiment = state.chat_engine.emotion.pleasure
            days = (time.time() - state.start_time) / 86400
            new_bond = state.bond_tracker.update(depth, sentiment, days)
            state.bond_tracker.save()
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "bond_tracker", "error": str(e)},
                source="bond_tracker", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="bond_tracker", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_intent_drift_hooks(event_bus, hook_manager, state):
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 10:
            return
        counter = 0
        if not state.intent_drift_detector:
            return
        try:
            alerts = state.intent_drift_detector.check_drift()
            for alert in alerts:
                await event_bus.publish(Event(
                    type="security.alert",
                    payload=alert,
                    source="intent_drift_detector", priority=Priority.HIGH,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "intent_drift_detector", "error": str(e)},
                source="intent_drift_detector", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="intent_drift", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_audit_hooks(event_bus, hook_manager, state):
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 10:
            return
        counter = 0
        if not state.audit_agent or not state.memory_store:
            return
        try:
            cursor = await state.memory_store.db.execute(
                "SELECT * FROM capsules WHERE source NOT IN ('audit_agent', 'self_reflection') ORDER BY timestamp DESC LIMIT 10"
            )
            rows = await cursor.fetchall()
            for row in rows:
                cap = dict(row)
                count = await state.audit_agent.extract_and_deposit(cap)
                if count:
                    await event_bus.publish(Event(
                        type="audit.triples_extracted",
                        payload={"capsule_id": cap.get("event_id"), "count": count},
                        source="audit_agent", priority=Priority.LOW,
                    ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "audit_agent", "error": str(e)},
                source="audit_agent", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="audit_agent", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_thinking_evolution_hooks(event_bus, hook_manager, state):
    async def on_hour(event):
        if not hasattr(state, 'thinking_evolver') or not state.thinking_evolver:
            return
        idle = time.time() - state.last_interaction_time
        if idle < 3600:
            return
        try:
            ev_events = state.thinking_evolver.evolve()
            if ev_events:
                await event_bus.publish(Event(
                    type="thinking.evolved",
                    payload={"events": ev_events},
                    source="thinking_evolver", priority=Priority.LOW,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "thinking_evolver", "error": str(e)},
                source="thinking_evolver", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="thinking_evolver", event_type="heartbeat.hour",
        callback=on_hour, timeout=30.0,
    ))


def register_milestone_hooks(event_bus, hook_manager, state):
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 5:
            return
        counter = 0
        if not hasattr(state, 'milestone_tracker') or not state.milestone_tracker:
            return
        try:
            mem_count = await state.memory_store.count() if state.memory_store else 0
            bond = state.bond_tracker.get_bond() if state.bond_tracker else 0.0
            days = (time.time() - state.start_time) / 86400
            triggered = state.milestone_tracker.check_milestones(
                memory_count=mem_count, bond_value=bond, days_since_birth=days
            )
            if triggered:
                await state.milestone_tracker.push_greetings(triggered)
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "milestones", "error": str(e)},
                source="milestones", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="milestones", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_hindsight_hooks(event_bus, hook_manager, state):
    """后见之明：每小时检查一次"""
    async def on_hour(event):
        if not hasattr(state, 'hindsight_engine') or not state.hindsight_engine:
            return
        if not state.chat_engine:
            return
        try:
            emotion = state.chat_engine.emotion
            triggered = await state.hindsight_engine.tick(emotion)
            if triggered:
                await event_bus.publish(Event(
                    type="hindsight.triggered",
                    payload={"count": len(triggered)},
                    source="hindsight_engine",
                    priority=Priority.LOW,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "hindsight_engine", "error": str(e)},
                source="hindsight_engine",
                priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="hindsight_engine",
        event_type="heartbeat.hour",
        callback=on_hour,
        timeout=30.0,
    ))