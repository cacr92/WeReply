import json
import os
import queue
import sys
import threading
import time
import uuid
from dataclasses import dataclass, field
from typing import Any, Dict, Optional, List, Tuple

VENDOR_ROOT = os.path.join(os.path.dirname(__file__), "vendor", "wxauto")
if os.path.isdir(VENDOR_ROOT) and VENDOR_ROOT not in sys.path:
    sys.path.insert(0, VENDOR_ROOT)

try:
    from wxauto import WeChat
except Exception:
    WeChat = None


ACK_TIMEOUT_SECONDS = 3
MAX_ACK_RETRIES = 3
DEFAULT_POLL_INTERVAL = 0.8
LISTEN_TARGET_KINDS = {"direct", "group", "unknown"}


@dataclass
class PendingMessage:
    envelope: Dict[str, Any]
    sent_at: float
    retries: int = 0


@dataclass
class AgentState:
    listening: bool = False
    poll_interval: float = DEFAULT_POLL_INTERVAL
    last_message_keys: Dict[str, str] = field(default_factory=dict)
    pending: Dict[str, PendingMessage] = field(default_factory=dict)
    wx: Optional[Any] = None
    wechat_ready: bool = False
    listen_targets: Dict[str, str] = field(default_factory=dict)
    active_targets: Dict[str, str] = field(default_factory=dict)
    active_kinds: Dict[str, str] = field(default_factory=dict)


STATE = AgentState()
COMMAND_QUEUE: "queue.Queue[Dict[str, Any]]" = queue.Queue()
MESSAGE_QUEUE: "queue.Queue[Tuple[Any, Any, str]]" = queue.Queue()


def send_json(message: Dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(message, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def normalize_listen_targets(raw_targets: Any) -> List[Dict[str, str]]:
    if not isinstance(raw_targets, list):
        return []
    normalized: List[Dict[str, str]] = []
    seen = set()
    for item in raw_targets:
        if not isinstance(item, dict):
            continue
        name = str(item.get("name", "")).strip()
        if not name or name in seen:
            continue
        kind = str(item.get("kind", "unknown")).strip().lower()
        if kind not in LISTEN_TARGET_KINDS:
            kind = "unknown"
        seen.add(name)
        normalized.append({"name": name, "kind": kind})
    return normalized


def select_wechat_main_hwnd(
    windows: list[tuple[int, str, str]],
    path_by_hwnd: Dict[int, str],
) -> Optional[int]:
    candidates = []
    for hwnd, class_name, title in windows:
        class_name = class_name or ""
        title = title or ""
        if (
            "微信" in title
            or "WeChat" in title
            or "Weixin" in title
            or "WeChat" in class_name
            or "Weixin" in class_name
        ):
            candidates.append((hwnd, class_name, title))
    if not candidates:
        return None

    for hwnd, _, _ in candidates:
        path = path_by_hwnd.get(hwnd) or ""
        if os.path.basename(path).lower() == "wechat.exe":
            return hwnd
    for hwnd, _, title in candidates:
        if title in ("微信", "WeChat"):
            return hwnd
    for hwnd, _, title in candidates:
        if title:
            return hwnd
    return candidates[0][0]


def find_wechat_main_hwnd() -> Optional[int]:
    try:
        from wxauto.utils import GetAllWindows, GetPathByHwnd
    except Exception:
        return None
    windows = GetAllWindows()
    path_by_hwnd: Dict[int, str] = {}
    for hwnd, _, _ in windows:
        try:
            path_by_hwnd[hwnd] = GetPathByHwnd(hwnd) or ""
        except Exception:
            path_by_hwnd[hwnd] = ""
    return select_wechat_main_hwnd(windows, path_by_hwnd)


def envelope(msg_type: str, payload: Dict[str, Any], msg_id: Optional[str] = None) -> Dict[str, Any]:
    return {
        "version": "1.0",
        "type": msg_type,
        "id": msg_id or str(uuid.uuid4()),
        "timestamp": int(time.time()),
        "payload": payload,
    }


def send_with_ack(msg_type: str, payload: Dict[str, Any]) -> None:
    env = envelope(msg_type, payload)
    send_json(env)
    STATE.pending[env["id"]] = PendingMessage(envelope=env, sent_at=time.time())


def send_ack(ack_id: str, ok: bool = True, error: str = "") -> None:
    send_json(envelope("event.ack", {"ack_id": ack_id, "ok": ok, "error": error}))


def emit_error(code: str, message: str, recoverable: bool = True) -> None:
    send_with_ack(
        "agent.error",
        {"code": code, "message": message, "recoverable": recoverable},
    )


def emit_status(state: str, detail: str = "") -> None:
    send_with_ack("agent.status", {"state": state, "detail": detail})


def ensure_wechat() -> Any:
    if STATE.wx is None:
        if WeChat is None:
            raise RuntimeError("wxauto 未安装")
        try:
            STATE.wx = WeChat()
        except Exception as exc:
            message = str(exc)
            if "未找到微信窗口" not in message and "未找到已登录的微信主窗口" not in message:
                raise
            fallback_hwnd = find_wechat_main_hwnd()
            if not fallback_hwnd:
                raise
            STATE.wx = WeChat(hwnd=fallback_hwnd)
    return STATE.wx


def get_current_chat_title(wx: Any) -> str:
    for attr in ("GetCurrentChat", "CurrentChat", "GetCurrentChatName", "GetChatName"):
        getter = getattr(wx, attr, None)
        if callable(getter):
            try:
                value = getter()
                if value:
                    return str(value)
            except Exception:
                continue
    return "unknown-chat"


def extract_message_text(message: Any) -> str:
    if isinstance(message, str):
        return message
    if isinstance(message, dict):
        for key in ("text", "content", "msg", "message"):
            value = message.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
    if isinstance(message, (list, tuple)) and message:
        value = message[-1]
        if isinstance(value, str):
            return value.strip()
    for attr in ("text", "content", "msg", "message"):
        value = getattr(message, attr, None)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return ""


def extract_sender_name(message: Any) -> str:
    if isinstance(message, dict):
        for key in ("sender_remark", "sender", "name", "from"):
            value = message.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
    for attr in ("sender_remark", "sender", "name", "from_user"):
        value = getattr(message, attr, None)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return ""


def extract_msg_id(message: Any) -> Optional[str]:
    if isinstance(message, dict):
        value = message.get("msg_id") or message.get("id")
        if value is not None:
            text = str(value).strip()
            if text:
                return text
    for attr in ("msg_id", "id"):
        value = getattr(message, attr, None)
        if value is not None:
            text = str(value).strip()
            if text:
                return text
    return None


def resolve_chat_title(chat: Any, fallback: str) -> str:
    try:
        info = chat.ChatInfo()
        if isinstance(info, dict):
            title = info.get("chat_name") or info.get("chat_remark")
            if isinstance(title, str) and title.strip():
                return title.strip()
    except Exception:
        pass
    return fallback


def resolve_is_group(chat: Any, kind: str) -> bool:
    if kind == "group":
        return True
    if kind == "direct":
        return False
    try:
        info = chat.ChatInfo()
        if isinstance(info, dict):
            return info.get("chat_type") == "group"
    except Exception:
        pass
    return False


def listen_callback(message: Any, chat: Any) -> None:
    if not STATE.listening:
        return
    chat_name = getattr(chat, "who", None) or str(chat)
    if not chat_name:
        chat_name = "unknown-chat"
    MESSAGE_QUEUE.put((message, chat, chat_name))


def handle_incoming_message(message: Any, chat: Any, chat_name: str) -> None:
    text = extract_message_text(message)
    if not text:
        return
    msg_id = extract_msg_id(message)
    msg_hash = getattr(message, "hash", None)
    key = msg_id or (str(msg_hash) if msg_hash else f"{extract_sender_name(message)}:{text}")
    if STATE.last_message_keys.get(chat_name) == key:
        return
    STATE.last_message_keys[chat_name] = key

    kind = STATE.active_kinds.get(chat_name, "unknown")
    chat_title = resolve_chat_title(chat, chat_name)
    payload = {
        "chat_id": chat_title,
        "chat_title": chat_title,
        "is_group": resolve_is_group(chat, kind),
        "sender_name": extract_sender_name(message) or chat_title,
        "text": text,
        "timestamp": int(time.time()),
        "msg_id": msg_id,
    }
    send_with_ack("message.new", payload)


def drain_message_queue(max_items: int = 50) -> None:
    for _ in range(max_items):
        try:
            message, chat, chat_name = MESSAGE_QUEUE.get_nowait()
        except queue.Empty:
            break
        handle_incoming_message(message, chat, chat_name)


def try_ensure_wechat() -> Optional[Any]:
    try:
        return ensure_wechat()
    except Exception as exc:
        emit_error("LISTEN_FAILED", str(exc), True)
        return None


def clear_message_queue() -> None:
    while True:
        try:
            MESSAGE_QUEUE.get_nowait()
        except queue.Empty:
            break


def clear_active_listeners(wx: Optional[Any]) -> None:
    if wx is not None:
        for actual in list(STATE.active_targets.values()):
            try:
                wx.RemoveListenChat(actual, close_window=True)
            except Exception:
                pass
    STATE.active_targets.clear()
    STATE.active_kinds.clear()


def reconcile_listeners(desired: Dict[str, str], allow_add: bool) -> None:
    wx = STATE.wx or try_ensure_wechat()
    if wx is None:
        return

    for target_name in list(STATE.active_targets.keys()):
        if target_name in desired:
            continue
        actual = STATE.active_targets.pop(target_name, None)
        if actual:
            STATE.active_kinds.pop(actual, None)
            try:
                wx.RemoveListenChat(actual, close_window=True)
            except Exception:
                pass

    if not allow_add:
        return

    for target_name, kind in desired.items():
        if target_name in STATE.active_targets:
            continue
        try:
            result = wx.AddListenChat(target_name, listen_callback)
        except Exception as exc:
            emit_error("LISTEN_TARGET_FAILED", f"{target_name}: {exc}", True)
            continue
        if not hasattr(result, "ChatInfo"):
            emit_error("LISTEN_TARGET_FAILED", f"{target_name}: 无法监听该会话", True)
            continue
        chat_name = getattr(result, "who", None) or target_name
        STATE.active_targets[target_name] = chat_name
        STATE.active_kinds[chat_name] = kind


def set_listen_targets(raw_targets: Any, allow_add: bool) -> None:
    normalized = normalize_listen_targets(raw_targets)
    desired = {item["name"]: item["kind"] for item in normalized}
    STATE.listen_targets = desired
    reconcile_listeners(desired, allow_add)


def write_input(chat_id: str, text: str, restore_clipboard: bool) -> None:
    try:
        wx = ensure_wechat()
    except Exception as exc:
        emit_error("WRITE_FAILED", str(exc), True)
        send_with_ack("input.result", {"ok": False, "error": str(exc)})
        return

    try:
        if hasattr(wx, "ChatWith"):
            wx.ChatWith(chat_id)
    except Exception:
        pass

    try:
        import pyperclip
        import pyautogui
    except Exception as exc:
        send_with_ack("input.result", {"ok": False, "error": str(exc)})
        return

    previous = None
    if restore_clipboard:
        try:
            previous = pyperclip.paste()
        except Exception:
            previous = None

    try:
        pyperclip.copy(text)
        pyautogui.hotkey("ctrl", "v")
        send_with_ack("input.result", {"ok": True, "error": ""})
    except Exception as exc:
        send_with_ack("input.result", {"ok": False, "error": str(exc)})
    finally:
        if restore_clipboard and previous is not None:
            try:
                pyperclip.copy(previous)
            except Exception:
                pass


def list_recent_chats() -> List[Dict[str, str]]:
    wx = STATE.wx or try_ensure_wechat()
    if wx is None:
        return []
    try:
        sessions = wx.GetSession()
    except Exception as exc:
        emit_error("CHAT_LIST_FAILED", str(exc), True)
        return []
    results: List[Dict[str, str]] = []
    for session in sessions or []:
        name = getattr(session, "name", None)
        if not isinstance(name, str):
            continue
        name = name.strip()
        if not name:
            continue
        results.append({"chat_id": name, "chat_title": name, "kind": "unknown"})
    return results


def handle_command(message: Dict[str, Any]) -> None:
    msg_type = message.get("type", "")
    msg_id = message.get("id", "")
    payload = message.get("payload", {})

    if msg_type == "event.ack":
        ack_id = payload.get("ack_id")
        if isinstance(ack_id, str) and ack_id in STATE.pending:
            STATE.pending.pop(ack_id, None)
        return

    if msg_id:
        send_ack(msg_id, True, "")

    if msg_type == "listen.start" or msg_type == "listen.resume":
        interval = payload.get("poll_interval_ms")
        if isinstance(interval, (int, float)) and interval >= 200:
            STATE.poll_interval = max(interval / 1000.0, 0.2)
        STATE.listening = True
        targets = payload.get("targets")
        if targets is not None:
            set_listen_targets(targets, True)
        else:
            reconcile_listeners(STATE.listen_targets, True)
        emit_status("listening", "")
        return

    if msg_type == "listen.pause":
        STATE.listening = False
        clear_message_queue()
        emit_status("paused", "")
        return

    if msg_type == "listen.stop":
        STATE.listening = False
        clear_message_queue()
        clear_active_listeners(STATE.wx)
        emit_status("idle", "")
        return

    if msg_type == "listen.targets":
        targets = payload.get("targets")
        set_listen_targets(targets, STATE.listening)
        return

    if msg_type == "input.write":
        chat_id = str(payload.get("chat_id", "")).strip()
        text = str(payload.get("text", "")).strip()
        restore = bool(payload.get("restore_clipboard", True))
        if not chat_id or not text:
            send_with_ack("input.result", {"ok": False, "error": "chat_id or text is empty"})
            return
        write_input(chat_id, text, restore)
        return

    if msg_type == "chats.list":
        request_id = str(payload.get("request_id", "")).strip()
        if not request_id:
            emit_error("CHAT_LIST_FAILED", "request_id missing", True)
        chats = list_recent_chats()
        send_with_ack("chats.list.result", {"request_id": request_id, "chats": chats})
        return


def read_stdin() -> None:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        COMMAND_QUEUE.put(msg)


def process_pending() -> None:
    now = time.time()
    for msg_id, pending in list(STATE.pending.items()):
        if now - pending.sent_at < ACK_TIMEOUT_SECONDS:
            continue
        if pending.retries >= MAX_ACK_RETRIES:
            STATE.pending.pop(msg_id, None)
            continue
        send_json(pending.envelope)
        pending.sent_at = now
        pending.retries += 1


def main() -> None:
    send_with_ack(
        "agent.ready",
        {
            "platform": "windows",
        "agent_version": "0.1.0",
            "capabilities": ["listen", "write", "chats.list"],
            "supports_clipboard_restore": True,
        },
    )

    reader = threading.Thread(target=read_stdin, daemon=True)
    reader.start()

    while True:
        try:
            message = COMMAND_QUEUE.get(timeout=0.1)
            handle_command(message)
        except queue.Empty:
            pass

        if STATE.listening:
            drain_message_queue()

        process_pending()


if __name__ == "__main__":
    main()
