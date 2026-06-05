import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Message {
  role: "user" | "assistant";
  content: string;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [status, setStatus] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke("get_status").then((s: any) => {
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    });
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const sendMessage = async () => {
    if (!input.trim()) return;

    const userMsg: Message = { role: "user", content: input };
    setMessages((prev) => [...prev, userMsg]);
    setInput("");

    try {
      const response = await invoke<string>("send_message", { text: input });
      const assistantMsg: Message = { role: "assistant", content: response };
      setMessages((prev) => [...prev, assistantMsg]);

      const s: any = await invoke("get_status");
      setStatus(`v${s.version} | ${s.turn_count} turns`);
    } catch (e: any) {
      const errorMsg: Message = {
        role: "assistant",
        content: `Error: ${e}`,
      };
      setMessages((prev) => [...prev, errorMsg]);
    }
  };

  const clearHistory = async () => {
    await invoke("clear_history");
    setMessages([]);
    const s: any = await invoke("get_status");
    setStatus(`v${s.version} | ${s.turn_count} turns`);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div className="app">
      <header className="header">
        <h1>Morn</h1>
        <span className="status">{status}</span>
        <button className="clear-btn" onClick={clearHistory}>
          Clear
        </button>
      </header>

      <main className="chat-area">
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            <div className="avatar">{msg.role === "user" ? "U" : "M"}</div>
            <div className="bubble">{msg.content}</div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </main>

      <footer className="input-bar">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type a message..."
          rows={1}
        />
        <button onClick={sendMessage} disabled={!input.trim()}>
          Send
        </button>
      </footer>
    </div>
  );
}

export default App;