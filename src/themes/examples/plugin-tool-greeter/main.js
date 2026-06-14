// plugin-tool-greeter entry point
class GreeterTool {
  constructor() {
    this.name = "greeter";
    this.description = "Greets a user by name";
  }

  async run(args) {
    const name = args.name || "World";
    return { ok: true, message: "Hello, " + name + "!" };
  }
}

export default GreeterTool;