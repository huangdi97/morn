// plugin-channel-echo entry point
class EchoChannel {
  constructor() {
    this.name = "echo";
    this._handler = null;
  }

  connect() {
    console.log("EchoChannel connected");
  }

  send(msg) {
    console.log("EchoChannel send:", msg);
    if (this._handler) {
      this._handler({ role: "assistant", content: "Echo: " + msg });
    }
  }

  onMessage(handler) {
    this._handler = handler;
  }

  disconnect() {
    console.log("EchoChannel disconnected");
  }
}

export default EchoChannel;