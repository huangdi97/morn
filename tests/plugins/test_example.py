from morn.plugins.example_hello import ExampleHelloPlugin


class TestExamplePlugin:
    def test_plugin_import(self):
        from morn.plugins.example_hello import ExampleHelloPlugin
        assert ExampleHelloPlugin is not None

    def test_example_plugin_id(self):
        plugin = ExampleHelloPlugin()
        assert plugin.plugin_id == "example_hello"