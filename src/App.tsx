import { ChangeEvent, useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { message, Modal } from "antd";
import "./App.css";
import type { ErrorPayload, Status, Suggestion, SuggestionsUpdated } from "./bindings";
import { commands } from "./bindings";
import type { ApiKeyStatus } from "./utils/apiKey";
import { getApiKeyStatusLabel, resolveApiKeySaveOutcome } from "./utils/apiKey";
import { getStyleLabel } from "./utils/labels";
import {
  DEFAULT_MODELS,
  normalizeModels,
  resolveModelSelection,
} from "./utils/models";
import { normalizeReplyText } from "./utils/reply";

const DEFAULT_STATUS: Status = {
  state: "idle",
  platform: "unknown",
  agent_connected: false,
  last_error: "",
};

function App() {
  const [status, setStatus] = useState<Status>(DEFAULT_STATUS);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [draftText, setDraftText] = useState("");
  const [apiKeySet, setApiKeySet] = useState(false);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [apiKeyStatus, setApiKeyStatus] = useState<ApiKeyStatus>("idle");
  const [lastChatId, setLastChatId] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [models, setModels] = useState<string[]>(DEFAULT_MODELS);
  const [selectedModel, setSelectedModel] = useState(DEFAULT_MODELS[0]);
  const [modelLoading, setModelLoading] = useState(false);

  useEffect(() => {
    const bootstrap = async () => {
      const [statusRes, keyRes, configRes] = await Promise.all([
        commands.getStatus(),
        commands.getApiKeyStatus(),
        commands.getConfig(),
      ]);
      if (statusRes.success && statusRes.data) {
        setStatus(statusRes.data);
      }
      if (keyRes.success && typeof keyRes.data === "boolean") {
        setApiKeySet(keyRes.data);
        setApiKeyStatus(keyRes.data ? "connected" : "idle");
      }
      if (configRes.success && configRes.data?.deepseek_model) {
        setSelectedModel(configRes.data.deepseek_model);
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

  const handleStart = useCallback(async () => {
    const res = await commands.startListening();
    if (res.success) {
      message.success("开始监听");
    } else {
      message.error(res.message || "启动失败");
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

  const handleInsertSuggestion = useCallback(
    async (suggestion: Suggestion) => {
      if (!lastChatId) {
        message.warning("暂无可写入的聊天");
        return;
      }
      const normalized = normalizeReplyText(suggestion.text);
      if (!normalized.ok) {
        message.warning(normalized.reason);
        return;
      }
      const res = await commands.writeSuggestion(lastChatId, normalized.text);
      if (res.success) {
        message.success("已写入输入框");
      } else {
        message.error(res.message || "写入失败");
      }
    },
    [lastChatId],
  );

  const handleWrite = useCallback(async () => {
    if (!lastChatId) {
      message.warning("暂无可写入的聊天");
      return;
    }
    const normalized = normalizeReplyText(draftText);
    if (!normalized.ok) {
      message.warning(normalized.reason);
      return;
    }
    const res = await commands.writeSuggestion(lastChatId, normalized.text);
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
    try {
      const res = await commands.saveApiKey(apiKeyInput.trim());
      const outcome = resolveApiKeySaveOutcome(res);
      setApiKeyStatus(outcome.status);
      setApiKeySet(outcome.apiKeySet);
      if (outcome.clearInput) {
        setApiKeyInput("");
      }
      if (outcome.status === "connected") {
        message.success(outcome.message);
        setModelLoading(true);
        const previousModel = selectedModel;
        try {
          const modelsRes = await commands.listModels();
          if (modelsRes.success && Array.isArray(modelsRes.data)) {
            const normalized = normalizeModels(modelsRes.data);
            setModels(normalized);
            const selection = resolveModelSelection(normalized, selectedModel);
            setSelectedModel(selection.selected);
            if (selection.changed) {
              const saveRes = await commands.setDeepseekModel(selection.selected);
              if (!saveRes.success) {
                message.error(saveRes.message || "模型保存失败");
                setSelectedModel(previousModel);
              }
            }
          } else {
            message.warning(modelsRes.message || "模型列表获取失败，使用默认模型");
            setModels(DEFAULT_MODELS);
          }
        } catch (err) {
          message.error("模型列表获取失败");
          setModels(DEFAULT_MODELS);
        } finally {
          setModelLoading(false);
        }
      } else {
        message.error(outcome.message);
      }
    } catch (err) {
      const outcome = resolveApiKeySaveOutcome(null, err);
      setApiKeyStatus(outcome.status);
      setApiKeySet(outcome.apiKeySet);
      message.error(outcome.message);
    }
  }, [apiKeyInput, selectedModel]);

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

  const handleModelChange = useCallback(
    async (event: ChangeEvent<HTMLSelectElement>) => {
      const nextModel = event.target.value;
      const previous = selectedModel;
      setSelectedModel(nextModel);
      const res = await commands.setDeepseekModel(nextModel);
      if (!res.success) {
        message.error(res.message || "模型保存失败");
        setSelectedModel(previous);
      }
    },
    [selectedModel],
  );

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
          <button className="ghost" onClick={() => setSettingsOpen(true)}>
            设置
          </button>
        </div>
      </header>

      <section className="controls">
        <button
          onClick={handleStart}
          disabled={status.state === "listening" || status.state === "generating"}
        >
          开始监听
        </button>
        <button className="ghost" onClick={handleStop} disabled={status.state === "idle"}>
          停止
        </button>
      </section>

      <section className="grid compact">
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
                  className="suggestion"
                  onClick={() => handleInsertSuggestion(item)}
                >
                  <span className="tag">{getStyleLabel(item.style)}</span>
                  <span className="text">{item.text}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="panel reply">
          <div className="panel-header">
            <h2>回复消息</h2>
          </div>
          <textarea
            placeholder="输入要写入的回复（仅写入微信输入框，不发送）"
            value={draftText}
            onChange={(event) => setDraftText(event.target.value)}
          />
          <div className="reply-actions">
            <button onClick={handleWrite} disabled={!draftText.trim()}>
              回复消息
            </button>
          </div>
        </div>
      </section>

      <Modal
        title="设置"
        open={settingsOpen}
        onCancel={() => setSettingsOpen(false)}
        footer={null}
      >
        <div className="panel settings">
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
        <div className="panel settings">
          <div className="panel-header">
            <h2>模型</h2>
            <span>{modelLoading ? "获取中" : "自动获取"}</span>
          </div>
          <div className="model-select">
            <select
              value={selectedModel}
              onChange={handleModelChange}
              disabled={modelLoading}
            >
              {models.map((model) => (
                <option key={model} value={model}>
                  {model}
                </option>
              ))}
            </select>
            <p>保存密钥后将刷新模型列表</p>
          </div>
        </div>
      </Modal>
    </main>
  );
}

export default App;
