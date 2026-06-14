import { useEffect, useState } from "react";

export default function MemoryManager() {
  const [memories, setMemories] = useState<string[]>([]);
  const [query, setQuery] = useState("");

  useEffect(() => {
    (async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const list = await invoke<string[]>("list_memories");
        setMemories(list);
      } catch (e) {
        console.error("Failed to list memories", e);
      }
    })();
  }, []);

  const handleSearch = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const results = await invoke<string[]>("search_memories", { q: query });
      setMemories(results);
    } catch (e) {
      console.error("Failed to search memories", e);
    }
  };

  return (
    <div className="memory-manager">
      <h2>Memory Manager</h2>
      <div className="memory-search">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search memories..."
        />
        <button onClick={handleSearch}>Search</button>
      </div>
      {memories.length === 0 ? (
        <p className="empty-state">No memories found.</p>
      ) : (
        <table className="memory-table">
          <thead>
            <tr>
              <th>Memory</th>
            </tr>
          </thead>
          <tbody>
            {memories.map((m, i) => (
              <tr key={i}>
                <td>{m}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}