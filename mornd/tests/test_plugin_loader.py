import pytest
from morn.kernel.plugin import MornPlugin, PluginDependency, PluginContext
from morn.kernel.plugin_loader import PluginLoader


class TestMornPlugin:
    def test_abstract_class_cannot_instantiate(self):
        with pytest.raises(TypeError):
            MornPlugin()

    def test_concrete_plugin_can_instantiate(self):
        class TestPlugin(MornPlugin):
            plugin_id = "test_plugin"
            name = "测试插件"

            async def on_load(self, context):
                pass

        plugin = TestPlugin()
        assert plugin.plugin_id == "test_plugin"
        assert plugin.version == "0.1.0"
        assert plugin.is_loaded is False


class TestPluginLoader:
    @pytest.mark.asyncio
    async def test_load_simple_plugin(self):
        class SimplePlugin(MornPlugin):
            plugin_id = "simple"
            name = "简单插件"

            async def on_load(self, context):
                pass

        loader = PluginLoader()
        loaded = await loader.load(SimplePlugin)
        assert loaded is not None
        assert loaded.plugin_id == "simple"
        assert loaded.is_loaded

    @pytest.mark.asyncio
    async def test_load_and_unload(self):
        class TempPlugin(MornPlugin):
            plugin_id = "temp"
            name = "临时插件"

            async def on_load(self, context):
                pass

        loader = PluginLoader()
        await loader.load(TempPlugin)
        assert loader.get_plugin("temp") is not None

        result = await loader.unload("temp")
        assert result is True
        assert loader.get_plugin("temp") is None

    @pytest.mark.asyncio
    async def test_list_plugins(self):
        class ListPlugin(MornPlugin):
            plugin_id = "listable"
            name = "可列表插件"

            async def on_load(self, context):
                pass

        loader = PluginLoader()
        await loader.load(ListPlugin)

        plugins = loader.list_plugins()
        assert len(plugins) >= 1
        listable = [p for p in plugins if p["plugin_id"] == "listable"]
        assert len(listable) == 1
        assert listable[0]["name"] == "可列表插件"
