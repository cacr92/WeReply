import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { message } from "antd";
import "./App.css";
import type {
  Config,
  ErrorPayload,
  Status,
  Suggestion,
  SuggestionsUpdated,
} from "./bindings";
import { commands } from "./bindings";
import { getStateLabel, getStyleLabel } from "./utils/labels";

const DEFAULT_STATUS: Status = {
  state: "idle",
  platform: "unknown",
  agent_connected: false,
  last_error: "",
};

const DEFAULT_CONFIG: Config = {
  deepseek_model: "deepseek-chat",
  suggestion_count: 3,
  context_max_messages: 10,
  context_max_chars: 2000,
  poll_interval_ms: 800,
  temperature: 0.7,
  top_p: 1.0,
  base_url: "https://api.deepseek.com",
  timeout_ms: 12000,
  max_retries: 2,
  log_level: "info",
  log_to_file: false,
};

function App() {
  const [status, setStatus] = useState<Status>(DEFAULT_STATUS);
  const [config, setConfig] = useState<Config>(DEFAULT_CONFIG);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [activeSuggestionId, setActiveSuggestionId] = useState<string | null>(
    null,
  );
  const [draftText, setDraftText] = useState("");
  const [apiKeySet, setApiKeySet] = useState(false);
  const [apiKeyInput, setApiKeyInput] = useState("");
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
      const [statusRes, configRes, keyRes] = await Promise.all([
        commands.getStatus(),
        commands.getConfig(),
        commands.getApiKeyStatus(),
      ]);
      if (statusRes.success && statusRes.data) {
        setStatus(statusRes.data);
      }
      if (configRes.success && configRes.data) {
        setConfig(configRes.data);
      }
      if (keyRes.success && typeof keyRes.data === "boolean") {
        setApiKeySet(keyRes.data);
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

  const updateConfigField = useCallback(
    <K extends keyof Config>(key: K, value: Config[K]) => {
      setConfig((prev) => ({ ...prev, [key]: value }));
    },
    [],
  );

  const handleSaveConfig = useCallback(async () => {
    const res = await commands.setConfig(config);
    if (res.success) {
      message.success("配置已保存");
    } else {
      message.error(res.message || "保存失败");
    }
  }, [config]);

  const handleSaveApiKey = useCallback(async () => {
    if (!apiKeyInput.trim()) {
      message.warning("请输入 API 密钥");
      return;
    }
    const res = await commands.saveApiKey(apiKeyInput.trim());
    if (res.success) {
      message.success("API 密钥已保存");
      setApiKeyInput("");
      setApiKeySet(true);
    } else {
      message.error(res.message || "保存失败");
    }
  }, [apiKeyInput]);

  const handleDeleteApiKey = useCallback(async () => {
    const res = await commands.deleteApiKey();
    if (res.success) {
      message.success("API 密钥已删除");
      setApiKeySet(false);
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

        <div className="panel config">
          <div className="panel-header">
            <h2>配置面板</h2>
            <span>DeepSeek / 监听参数</span>
          </div>
          <div className="config-grid">
            <label>
              模型
              <input
                value={config.deepseek_model}
                onChange={(event) =>
                  updateConfigField("deepseek_model", event.target.value)
                }
              />
            </label>
            <label>
              Base URL
              <input
                value={config.base_url}
                onChange={(event) =>
                  updateConfigField("base_url", event.target.value)
                }
              />
            </label>
            <label>
              temperature
              <input
                type="number"
                step="0.1"
                value={config.temperature}
                onChange={(event) =>
                  updateConfigField(
                    "temperature",
                    Number.parseFloat(event.target.value || "0"),
                  )
                }
              />
            </label>
            <label>
              top_p
              <input
                type="number"
                step="0.1"
                value={config.top_p}
                onChange={(event) =>
                  updateConfigField(
                    "top_p",
                    Number.parseFloat(event.target.value || "0"),
                  )
                }
              />
            </label>
            <label>
              建议数量
              <input
                type="number"
                value={config.suggestion_count}
                onChange={(event) =>
                  updateConfigField(
                    "suggestion_count",
                    Number.parseInt(event.target.value || "0", 10),
                  )
                }
              />
            </label>
            <label>
              上下文条数
              <input
                type="number"
                value={config.context_max_messages}
                onChange={(event) =>
                  updateConfigField(
                    "context_max_messages",
                    Number.parseInt(event.target.value || "0", 10),
                  )
                }
              />
            </label>
            <label>
              上下文字符上限
              <input
                type="number"
                value={config.context_max_chars}
                onChange={(event) =>
                  updateConfigField(
                    "context_max_chars",
                    Number.parseInt(event.target.value || "0", 10),
                  )
                }
              />
            </label>
            <label>
              监听间隔 (ms)
              <input
                type="number"
                value={config.poll_interval_ms}
                onChange={(event) =>
                  updateConfigField(
                    "poll_interval_ms",
                    Number.parseInt(event.target.value || "0", 10),
                  )
                }
              />
            </label>
            <label>
              超时 (ms)
              <input
                type="number"
                value={config.timeout_ms}
                onChange={(event) =>
                  updateConfigField(
                    "timeout_ms",
                    Number.parseInt(event.target.value || "0", 10),
                  )
                }
              />
            </label>
            <label>
              最大重试
              <input
                type="number"
                value={config.max_retries}
                onChange={(event) =>
                  updateConfigField(
                    "max_retries",
                    Number.parseInt(event.target.value || "0", 10),
                  )
                }
              />
            </label>
            <label>
              日志级别
              <select
                value={config.log_level}
                onChange={(event) =>
                  updateConfigField("log_level", event.target.value)
                }
              >
                <option value="trace">trace</option>
                <option value="debug">debug</option>
                <option value="info">info</option>
                <option value="warn">warn</option>
                <option value="error">error</option>
              </select>
            </label>
            <label className="toggle">
              <input
                type="checkbox"
                checked={config.log_to_file}
                onChange={(event) =>
                  updateConfigField("log_to_file", event.target.checked)
                }
              />
              写入日志文件
            </label>
          </div>
          <div className="config-actions">
            <button onClick={handleSaveConfig}>保存配置</button>
          </div>
        </div>

        <div className="panel security">
          <div className="panel-header">
            <h2>API 密钥</h2>
            <span>{apiKeySet ? "已保存" : "未设置"}</span>
          </div>
          <div className="api-key">
            <input
              type="password"
              placeholder="sk-..."
              value={apiKeyInput}
              onChange={(event) => setApiKeyInput(event.target.value)}
            />
            <button onClick={handleSaveApiKey}>保存密钥</button>
            <button className="ghost" onClick={handleDeleteApiKey}>
              删除密钥
            </button>
          </div>
        </div>
      </section>
    </main>
  );
}

export default App;
