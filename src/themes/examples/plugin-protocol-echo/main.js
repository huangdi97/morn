// plugin-protocol-echo entry point
class EchoProtocol {
  constructor() {
    this.name = "echo-protocol";
  }

  handleRequest(req) {
    if (req.path === "/ping") return { ok: true, data: "pong" };
    if (req.path === "/echo") return { ok: true, data: req.body };
    return { ok: false, error: "unknown endpoint" };
  }
}

export default EchoProtocol;