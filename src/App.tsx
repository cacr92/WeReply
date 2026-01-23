import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { message } from "antd";
import "./App.css";
import type { ErrorPayload, Status, Suggestion, SuggestionsUpdated } from "./bindings";
import { commands } from "./bindings";
import type { ApiKeyStatus } from "./utils/apiKey";
import { getApiKeyStatusLabel } from "./utils/apiKey";
import { getStateLabel, getStyleLabel } from "./utils/labels";

const DEFAULT_STATUS: Status = {
  state: "idle",
  platform: "unknown",
  agent_connected: false,
  last_error: "",
};

function App() {
  const [status, setStatus] = useState<Status>(DEFAULT_STATUS);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [activeSuggestionId, setActiveSuggestionId] = useState<string | null>(
    null,
  );
  const [draftText, setDraftText] = useState("");
  const [apiKeySet, setApiKeySet] = useState(false);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [apiKeyStatus, setApiKeyStatus] = useState<ApiKeyStatus>("idle");
  const [lastChatId, setLastChatId] = useState<string | null>(null);
  const [theme, setTheme] = useState(() => {
    const saved = localStorage.getItem("wereply-theme");
    return saved === "dark" ? "dark" : "light";
  });

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("wereply-theme", theme);
  }, [theme]);

  useEffect(() => {
    const bootstrap = async () => {
      const [statusRes, keyRes] = await Promise.all([
        commands.getStatus(),
        commands.getApiKeyStatus(),
      ]);
      if (statusRes.success && statusRes.data) {
        setStatus(statusRes.data);
      }
      if (keyRes.success && typeof keyRes.data === "boolean") {
        setApiKeySet(keyRes.data);
        setApiKeyStatus(keyRes.data ? "connected" : "idle");
      }
    };
    void bootstrap();
  }, []);

  useEffect(() => {
    const unlistenStatus = listen<Status>("status.changed", (event) => {
      setStatus(event.payload);
    });
    const unlistenSuggestions = listen<SuggestionsUpdated>(
      "suggestions.updated",
      (event) => {
        setSuggestions(event.payload.suggestions);
        setLastChatId(event.payload.chat_id);
        if (event.payload.suggestions[0]) {
          setActiveSuggestionId(event.payload.suggestions[0].id);
          setDraftText(event.payload.suggestions[0].text);
        }
      },
    );
    const unlistenError = listen<ErrorPayload>("error.raised", (event) => {
      message.error(event.payload.message);
    });

    return () => {
      void unlistenStatus.then((fn) => fn());
      void unlistenSuggestions.then((fn) => fn());
      void unlistenError.then((fn) => fn());
    };
  }, []);

  const activeSuggestion = useMemo(
    () => suggestions.find((item) => item.id === activeSuggestionId) || null,
    [activeSuggestionId, suggestions],
  );

  const handleStart = useCallback(async () => {
    const res = await commands.startListening();
    if (res.success) {
      message.success("开始监听");
    } else {
      message.error(res.message || "启动失败");
    }
  }, []);

  const handlePause = useCallback(async () => {
    const res = await commands.pauseListening();
    if (res.success) {
      message.success("已暂停");
    } else {
      message.error(res.message || "暂停失败");
    }
  }, []);

  const handleResume = useCallback(async () => {
    const res = await commands.resumeListening();
    if (res.success) {
      message.success("恢复监听");
    } else {
      message.error(res.message || "恢复失败");
    }
  }, []);

  const handleStop = useCallback(async () => {
    const res = await commands.stopListening();
    if (res.success) {
      message.success("已停止");
    } else {
      message.error(res.message || "停止失败");
    }
  }, []);

  const handleSelectSuggestion = useCallback(
    (suggestion: Suggestion) => {
      setActiveSuggestionId(suggestion.id);
      setDraftText(suggestion.text);
    },
    [setActiveSuggestionId],
  );

  const handleWrite = useCallback(async () => {
    if (!lastChatId) {
      message.warning("暂无可写入的聊天");
      return;
    }
    const res = await commands.writeSuggestion(lastChatId, draftText.trim());
    if (res.success) {
      message.success("已写入输入框");
    } else {
      message.error(res.message || "写入失败");
    }
  }, [draftText, lastChatId]);

  const handleSaveApiKey = useCallback(async () => {
    if (!apiKeyInput.trim()) {
      message.warning("请输入 API 密钥");
      return;
    }
    setApiKeyStatus("connecting");
    const res = await commands.saveApiKey(apiKeyInput.trim());
    if (res.success) {
      message.success("API 密钥已保存并连接成功");
      setApiKeyInput("");
      setApiKeySet(true);
      setApiKeyStatus("connected");
    } else {
      message.error(res.message || "连接失败");
      setApiKeySet(false);
      setApiKeyStatus("failed");
    }
  }, [apiKeyInput]);

  const handleDeleteApiKey = useCallback(async () => {
    const res = await commands.deleteApiKey();
    if (res.success) {
      message.success("API 密钥已删除");
      setApiKeySet(false);
      setApiKeyStatus("idle");
    } else {
      message.error(res.message || "删除失败");
    }
  }, []);

  return (
    <main className="app">
      <header className="topbar">
        <div className="brand">
          <div className="logo">WR</div>
          <div>
            <h1>WeReply</h1>
            <p>桌面端智能回复建议助手</p>
          </div>
        </div>
        <div className="top-actions">
          <button
            className="ghost"
            onClick={() => setTheme(theme === "light" ? "dark" : "light")}
          >
            {theme === "light" ? "夜间" : "日间"}主题
          </button>
          <div className="status-pill" data-state={status.state}>
            {getStateLabel(status.state)}
          </div>
        </div>
      </header>

      <section className="status-card">
        <div>
          <span className="label">平台</span>
          <strong>{status.platform}</strong>
        </div>
        <div>
          <span className="label">Agent</span>
          <strong>{status.agent_connected ? "已连接" : "未连接"}</strong>
        </div>
        <div>
          <span className="label">最近错误</span>
          <strong>{status.last_error || "暂无"}</strong>
        </div>
        <div className="actions">
          <button onClick={handleStart}>开始监听</button>
          <button
            className="ghost"
            onClick={handlePause}
            disabled={status.state !== "listening"}
          >
            暂停
          </button>
          <button
            className="ghost"
            onClick={handleResume}
            disabled={status.state !== "paused"}
          >
            继续
          </button>
          <button
            className="ghost"
            onClick={handleStop}
            disabled={status.state === "idle"}
          >
            停止
          </button>
        </div>
      </section>

      <section className="grid">
        <div className="panel suggestions">
          <div className="panel-header">
            <h2>回复建议</h2>
            <span>{suggestions.length} 条</span>
          </div>
          {suggestions.length === 0 ? (
            <div className="empty">等待新消息触发建议</div>
          ) : (
            <div className="suggestion-list">
              {suggestions.map((item) => (
                <button
                  key={item.id}
                  className={
                    item.id === activeSuggestionId
                      ? "suggestion active"
                      : "suggestion"
                  }
                  onClick={() => handleSelectSuggestion(item)}
                >
                  <span className="tag">{getStyleLabel(item.style)}</span>
                  <span className="text">{item.text}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="panel editor">
          <div className="panel-header">
            <h2>编辑区</h2>
            <span>{activeSuggestion ? "已选中建议" : "未选中"}</span>
          </div>
          <textarea
            placeholder="选择建议后可在此编辑内容"
            value={draftText}
            onChange={(event) => setDraftText(event.target.value)}
          />
          <div className="editor-actions">
            <button onClick={handleWrite} disabled={!draftText.trim()}>
              写入输入框
            </button>
          </div>
          <div className="preview">
            <span className="label">预览</span>
            <p>{draftText || "暂无内容"}</p>
          </div>
        </div>

        <div className="panel security">
          <div className="panel-header">
            <h2>API 密钥</h2>
            <span>{getApiKeyStatusLabel(apiKeyStatus)}</span>
          </div>
          <div className="api-key">
            <input
              type="password"
              placeholder="sk-..."
              value={apiKeyInput}
              onChange={(event) => setApiKeyInput(event.target.value)}
            />
            <button onClick={handleSaveApiKey} disabled={apiKeyStatus === "connecting"}>
              保存并连接
            </button>
            {apiKeySet ? (
              <button className="ghost" onClick={handleDeleteApiKey}>
                删除密钥
              </button>
            ) : null}
          </div>
        </div>
      </section>
    </main>
  );
}

export default App;
