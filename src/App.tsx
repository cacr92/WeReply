import { ChangeEvent, useCallback, useEffect, useReducer, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { message, Modal } from "antd";
import "./App.css";
import type {
  DeepseekDiagnostics,
  ErrorPayload,
  Status,
  Suggestion,
  SuggestionsUpdated,
} from "./bindings";
import { commands } from "./bindings";
import type { ApiKeyStatus } from "./utils/apiKey";
import { getApiKeyStatusLabel, resolveApiKeySaveOutcome } from "./utils/apiKey";
import { getApiKeyInputType, getApiKeyToggleLabel } from "./utils/apiKeyVisibility";
import { summarizeDiagnostics } from "./utils/diagnostics";
import { getStyleLabel } from "./utils/labels";
import {
  DEFAULT_MODELS,
  normalizeModels,
  resolveModelSelection,
} from "./utils/models";
import {
  ListenTarget,
  ListenTargetKind,
  MAX_LISTEN_TARGETS,
  mergeListenTargets,
  normalizeListenTargetList,
  normalizeListenTargets,
} from "./utils/listenTargets";
import { normalizeReplyText } from "./utils/reply";
import { createStatusState, statusReducer } from "./utils/status";

const DEFAULT_STATUS: Status = {
  state: "idle",
  platform: "unknown",
  agent_connected: false,
  last_error: "",
};

type RecentChat = {
  chat_id: string;
  chat_title: string;
  kind: ListenTargetKind;
};

const LISTEN_KIND_LABELS: Record<ListenTargetKind, string> = {
  direct: "私聊",
  group: "群聊",
  unknown: "未知",
};

function App() {
  const [statusState, dispatchStatus] = useReducer(
    statusReducer,
    DEFAULT_STATUS,
    createStatusState,
  );
  const status = statusState.status;
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [apiKeySet, setApiKeySet] = useState(false);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [apiKeyStatus, setApiKeyStatus] = useState<ApiKeyStatus>("idle");
  const [apiKeyVisible, setApiKeyVisible] = useState(false);
  const [apiKeyError, setApiKeyError] = useState<string | null>(null);
  const [lastChatId, setLastChatId] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [listenTargets, setListenTargets] = useState<ListenTarget[]>([]);
  const [listenInput, setListenInput] = useState("");
  const [listenKind, setListenKind] = useState<ListenTargetKind>("unknown");
  const [listenDirty, setListenDirty] = useState(false);
  const [recentChats, setRecentChats] = useState<RecentChat[]>([]);
  const [recentLoading, setRecentLoading] = useState(false);
  const [models, setModels] = useState<string[]>(DEFAULT_MODELS);
  const [selectedModel, setSelectedModel] = useState(DEFAULT_MODELS[0]);
  const [modelLoading, setModelLoading] = useState(false);
  const [diagnostics, setDiagnostics] = useState<DeepseekDiagnostics | null>(null);
  const [diagnosing, setDiagnosing] = useState(false);
  const [diagnosticsError, setDiagnosticsError] = useState<string | null>(null);
  const diagnosticsSummary = summarizeDiagnostics(diagnostics, diagnosticsError || undefined);

  useEffect(() => {
    const bootstrap = async () => {
      const [statusRes, keyRes, configRes, targetsRes] = await Promise.all([
        commands.getStatus(),
        commands.getApiKeyStatus(),
        commands.getConfig(),
        commands.getListenTargets(),
      ]);
      if (statusRes.success && statusRes.data) {
        dispatchStatus({ type: "bootstrap", status: statusRes.data });
      }
      if (keyRes.success && typeof keyRes.data === "boolean") {
        setApiKeySet(keyRes.data);
        setApiKeyStatus(keyRes.data ? "connected" : "idle");
      }
      if (configRes.success && configRes.data?.deepseek_model) {
        setSelectedModel(configRes.data.deepseek_model);
      }
      if (targetsRes.success && Array.isArray(targetsRes.data)) {
        const normalized = normalizeListenTargetList(targetsRes.data);
        setListenTargets(normalized);
        setListenDirty(false);
      }
    };
    void bootstrap();
  }, []);

  useEffect(() => {
    const unlistenStatus = listen<Status>("status.changed", (event) => {
      dispatchStatus({ type: "event", status: event.payload });
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

  const refreshRecentChats = useCallback(async () => {
    setRecentLoading(true);
    try {
      const res = await commands.listRecentChats();
      if (res.success && Array.isArray(res.data)) {
        setRecentChats(res.data as RecentChat[]);
      } else {
        message.error(res.message || "会话列表获取失败");
      }
    } catch (err) {
      message.error("会话列表获取失败");
    }
    setRecentLoading(false);
  }, []);

  useEffect(() => {
    if (!settingsOpen) {
      return;
    }
    void refreshRecentChats();
  }, [settingsOpen, refreshRecentChats]);

  const saveListenTargets = useCallback(
    async (targets: ListenTarget[], showToast: boolean) => {
      const normalized = normalizeListenTargetList(targets);
      if (normalized.length > MAX_LISTEN_TARGETS) {
        message.warning(`监听对象最多 ${MAX_LISTEN_TARGETS} 个`);
        return false;
      }
      const res = await commands.setListenTargets(normalized);
      if (res.success) {
        setListenTargets(normalized);
        setListenDirty(false);
        if (showToast) {
          message.success("监听对象已保存");
        }
        return true;
      }
      message.error(res.message || "监听对象保存失败");
      return false;
    },
    [setListenTargets, setListenDirty],
  );

  const handleAddManualTarget = useCallback(() => {
    const additions = normalizeListenTargets([listenInput], listenKind);
    if (additions.length === 0) {
      message.warning("请输入联系人或群名称");
      return;
    }
    const merged = mergeListenTargets(listenTargets, additions);
    if (merged.length === listenTargets.length) {
      message.info("已在监听列表中");
      return;
    }
    if (merged.length > MAX_LISTEN_TARGETS) {
      message.warning(`监听对象最多 ${MAX_LISTEN_TARGETS} 个`);
      return;
    }
    setListenTargets(merged);
    setListenInput("");
    setListenDirty(true);
  }, [listenInput, listenKind, listenTargets]);

  const handleAddRecentTarget = useCallback(
    (chat: RecentChat) => {
      const name = chat.chat_title.trim() || chat.chat_id.trim();
      if (!name) {
        return;
      }
      const merged = mergeListenTargets(listenTargets, [
        { name, kind: chat.kind },
      ]);
      if (merged.length === listenTargets.length) {
        message.info("已在监听列表中");
        return;
      }
      if (merged.length > MAX_LISTEN_TARGETS) {
        message.warning(`监听对象最多 ${MAX_LISTEN_TARGETS} 个`);
        return;
      }
      setListenTargets(merged);
      setListenDirty(true);
    },
    [listenTargets],
  );

  const handleRemoveTarget = useCallback((name: string) => {
    setListenTargets((prev) => prev.filter((item) => item.name !== name));
    setListenDirty(true);
  }, []);

  const handleSaveTargets = useCallback(async () => {
    void saveListenTargets(listenTargets, true);
  }, [listenTargets, saveListenTargets]);

  const handleStart = useCallback(async () => {
    if (listenTargets.length === 0) {
      message.warning("请先选择监听对象");
      setSettingsOpen(true);
      return;
    }
    if (listenDirty) {
      const saved = await saveListenTargets(listenTargets, false);
      if (!saved) {
        return;
      }
    }
    const res = await commands.startListening();
    if (res.success) {
      dispatchStatus({ type: "optimistic", state: "listening", last_error: "" });
      message.success("开始监听");
    } else {
      message.error(res.message || "启动失败");
    }
  }, [listenDirty, listenTargets, saveListenTargets, setSettingsOpen]);

  const handleStop = useCallback(async () => {
    const res = await commands.stopListening();
    if (res.success) {
      dispatchStatus({ type: "optimistic", state: "idle", last_error: "" });
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
      setApiKeyError(outcome.status === "failed" ? outcome.message : null);
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
      setApiKeyError(outcome.message);
      message.error(outcome.message);
    }
  }, [apiKeyInput, selectedModel]);

  const handleDeleteApiKey = useCallback(async () => {
    const res = await commands.deleteApiKey();
    if (res.success) {
      message.success("API 密钥已删除");
      setApiKeySet(false);
      setApiKeyStatus("idle");
      setDiagnostics(null);
      setApiKeyError(null);
    } else {
      message.error(res.message || "删除失败");
    }
  }, []);

  const handleDiagnose = useCallback(async () => {
    const trimmed = apiKeyInput.trim();
    if (!trimmed && !apiKeySet) {
      message.warning("请先输入或保存 API 密钥");
      return;
    }
    setDiagnosing(true);
    try {
      const res = await commands.diagnoseDeepseek(trimmed || undefined);
      if (res.success && res.data) {
        setDiagnostics(res.data);
        setDiagnosticsError(null);
        const summary = summarizeDiagnostics(res.data);
        if (summary.ok) {
          message.success(summary.message);
        } else {
          message.error(summary.message);
        }
      } else {
        const messageText = res.message || "连接诊断失败";
        setDiagnostics(null);
        setDiagnosticsError(messageText);
        message.error(messageText);
      }
    } catch (err) {
      const fallback = err instanceof Error ? err.message : "连接诊断失败";
      setDiagnostics(null);
      setDiagnosticsError(fallback);
      message.error(fallback);
    } finally {
      setDiagnosing(false);
    }
  }, [apiKeyInput, apiKeySet]);

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
        <div className="control-group">
          <button
            className="primary"
            onClick={handleStart}
            disabled={status.state === "listening" || status.state === "generating"}
          >
            开始监听
          </button>
          <button className="ghost" onClick={handleStop} disabled={status.state === "idle"}>
            停止
          </button>
        </div>
        <div className="top-actions">
          <button
            className="ghost icon-button"
            onClick={() => setSettingsOpen(true)}
            aria-label="设置"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M12 8.2a3.8 3.8 0 1 0 0 7.6 3.8 3.8 0 0 0 0-7.6Zm9.1 3.3-.9-.2a7.2 7.2 0 0 0-.7-1.7l.6-.7a1.2 1.2 0 0 0 0-1.7l-1.3-1.3a1.2 1.2 0 0 0-1.7 0l-.7.6c-.6-.3-1.2-.5-1.7-.7l-.2-.9a1.2 1.2 0 0 0-1.2-.9h-1.8a1.2 1.2 0 0 0-1.2.9l-.2.9c-.6.2-1.1.4-1.7.7l-.7-.6a1.2 1.2 0 0 0-1.7 0L4.4 6.3a1.2 1.2 0 0 0 0 1.7l.6.7c-.3.6-.5 1.1-.7 1.7l-.9.2a1.2 1.2 0 0 0-.9 1.2v1.8c0 .6.4 1.1.9 1.2l.9.2c.2.6.4 1.1.7 1.7l-.6.7a1.2 1.2 0 0 0 0 1.7l1.3 1.3a1.2 1.2 0 0 0 1.7 0l.7-.6c.6.3 1.1.5 1.7.7l.2.9c.1.6.6 1 1.2 1h1.8a1.2 1.2 0 0 0 1.2-1l.2-.9c.6-.2 1.1-.4 1.7-.7l.7.6a1.2 1.2 0 0 0 1.7 0l1.3-1.3a1.2 1.2 0 0 0 0-1.7l-.6-.7c.3-.6.5-1.1.7-1.7l.9-.2c.6-.1 1-.6 1-1.2v-1.8c0-.6-.4-1.1-1-1.2Z"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.6"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        </div>
      </header>

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
      </section>

      <Modal
        title="设置"
        open={settingsOpen}
        onCancel={() => setSettingsOpen(false)}
        footer={null}
        width={720}
        style={{ top: 24 }}
        styles={{
          body: {
            maxHeight: "calc(100vh - 180px)",
            overflowY: "auto",
          },
        }}
      >
        <div className="panel settings">
          <div className="panel-header">
            <h2>API 密钥</h2>
            <span>{getApiKeyStatusLabel(apiKeyStatus)}</span>
          </div>
          <div className="api-key">
            <input
              type={getApiKeyInputType(apiKeyVisible)}
              placeholder="sk-..."
              value={apiKeyInput}
              onChange={(event) => setApiKeyInput(event.target.value)}
            />
            <button
              className="ghost api-key-toggle"
              onClick={() => setApiKeyVisible((prev) => !prev)}
            >
              {getApiKeyToggleLabel(apiKeyVisible)}
            </button>
            <button onClick={handleSaveApiKey} disabled={apiKeyStatus === "connecting"}>
              保存并连接
            </button>
            <button className="ghost" onClick={handleDiagnose} disabled={diagnosing}>
              {diagnosing ? "诊断中..." : "连接诊断"}
            </button>
            {apiKeySet ? (
              <button className="ghost" onClick={handleDeleteApiKey}>
                删除密钥
              </button>
            ) : null}
            {apiKeyError ? <p className="api-error">{apiKeyError}</p> : null}
            {(diagnostics || diagnosticsError) && diagnosticsSummary ? (
              <div className="diagnostics">
                <p>{diagnosticsSummary.message}</p>
                <ul>
                  {diagnostics ? <li>Base URL：{diagnostics.base_url}</li> : null}
                  {diagnostics ? <li>模型：{diagnostics.model}</li> : null}
                  {diagnosticsSummary.lines.map((line) => (
                    <li key={line}>{line}</li>
                  ))}
                </ul>
              </div>
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
        <div className="panel settings">
          <div className="panel-header">
            <h2>监听对象</h2>
            <span>
              {listenTargets.length}/{MAX_LISTEN_TARGETS}
            </span>
          </div>
          <div className="listen-targets">
            <div className="listen-row">
              <input
                type="text"
                placeholder="输入联系人或群名"
                value={listenInput}
                onChange={(event) => setListenInput(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    handleAddManualTarget();
                  }
                }}
              />
              <select
                value={listenKind}
                onChange={(event) =>
                  setListenKind(event.target.value as ListenTargetKind)
                }
              >
                <option value="unknown">未知</option>
                <option value="direct">私聊</option>
                <option value="group">群聊</option>
              </select>
              <button className="small" onClick={handleAddManualTarget}>
                添加
              </button>
              <button
                className="ghost small"
                onClick={refreshRecentChats}
                disabled={recentLoading}
              >
                {recentLoading ? "刷新中..." : "刷新会话"}
              </button>
              <button
                className="small"
                onClick={handleSaveTargets}
                disabled={!listenDirty}
              >
                保存
              </button>
            </div>
            <div className="listen-columns">
              <div>
                <div className="listen-subtitle">已选择</div>
                {listenTargets.length === 0 ? (
                  <div className="empty">未选择任何对象</div>
                ) : (
                  <div className="listen-list">
                    {listenTargets.map((target) => (
                      <div className="listen-item" key={target.name}>
                        <div className="listen-meta">
                          <span className="listen-name">{target.name}</span>
                          <span className="listen-kind">
                            {LISTEN_KIND_LABELS[target.kind]}
                          </span>
                        </div>
                        <button
                          className="ghost small"
                          onClick={() => handleRemoveTarget(target.name)}
                        >
                          移除
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
              <div>
                <div className="listen-subtitle">最近会话</div>
                {recentLoading ? (
                  <div className="empty">加载中...</div>
                ) : recentChats.length === 0 ? (
                  <div className="empty">暂无会话</div>
                ) : (
                  <div className="listen-list">
                    {recentChats.map((chat) => (
                      <div
                        className="listen-item"
                        key={`${chat.chat_id}-${chat.chat_title}`}
                      >
                        <div className="listen-meta">
                          <span className="listen-name">{chat.chat_title}</span>
                          <span className="listen-kind">
                            {LISTEN_KIND_LABELS[chat.kind]}
                          </span>
                        </div>
                        <button
                          className="ghost small"
                          onClick={() => handleAddRecentTarget(chat)}
                        >
                          添加
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </Modal>
    </main>
  );
}

export default App;
