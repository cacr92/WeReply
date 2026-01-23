import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type Status = {
  state: string;
  platform: string;
  agent_connected: boolean;
  last_error: string;
};

type Suggestion = {
  id: string;
  style: string;
  text: string;
};

function App() {
  const [status, setStatus] = useState<Status>({
    state: "idle",
    platform: "unknown",
    agent_connected: false,
    last_error: "",
  });
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);

  useEffect(() => {
    void (async () => {
      const next = await invoke<Status>("get_status");
      setStatus(next);
    })();
  }, []);

  const handleStart = async () => {
    await invoke<boolean>("start_listening");
    setStatus((prev) => ({ ...prev, state: "listening" }));
    setSuggestions([
      { id: "s1", style: "正式", text: "好的，我看一下再回复你。" },
      { id: "s2", style: "中性", text: "收到，我稍后回复。" },
      { id: "s3", style: "轻松", text: "没问题，我等下回你～" },
    ]);
  };

  const handleWrite = async (suggestion: Suggestion) => {
    await invoke<boolean>("write_suggestion", {
      chat_id: "demo",
      text: suggestion.text,
    });
  };

  return (
    <main className="container">
      <header className="header">
        <h1>WeReply</h1>
        <div className="status">
          <span>状态：{status.state}</span>
          <span>平台：{status.platform}</span>
          <span>Agent：{status.agent_connected ? "已连接" : "未连接"}</span>
        </div>
      </header>

      <section className="actions">
        <button onClick={handleStart}>开始监听</button>
      </section>

      <section className="suggestions">
        <h2>回复建议</h2>
        {suggestions.length === 0 ? (
          <p className="muted">暂无建议</p>
        ) : (
          <ul>
            {suggestions.map((suggestion) => (
              <li key={suggestion.id}>
                <div className="suggestion-meta">{suggestion.style}</div>
                <div className="suggestion-text">{suggestion.text}</div>
                <button onClick={() => handleWrite(suggestion)}>写入输入框</button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}

export default App;
