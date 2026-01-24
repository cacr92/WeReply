import json
import os
import queue
import sys
import threading
import time
import uuid
from dataclasses import dataclass, field
from typing import Any, Dict, Optional

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


STATE = AgentState()
COMMAND_QUEUE: "queue.Queue[Dict[str, Any]]" = queue.Queue()


def send_json(message: Dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(message, ensure_ascii=False) + "\n")
    sys.stdout.flush()


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
        STATE.wx = WeChat()
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
        for key in ("sender", "name", "from"):
            value = message.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
    for attr in ("sender", "name", "from_user"):
        value = getattr(message, attr, None)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return ""


def extract_msg_id(message: Any) -> Optional[str]:
    if isinstance(message, dict):
        value = message.get("msg_id") or message.get("id")
        if isinstance(value, str) and value.strip():
            return value.strip()
    value = getattr(message, "msg_id", None)
    if isinstance(value, str) and value.strip():
        return value.strip()
    return None


def poll_messages() -> None:
    try:
        wx = ensure_wechat()
    except Exception as exc:
        emit_error("LISTEN_FAILED", str(exc), True)
        STATE.listening = False
        emit_status("error", "listen initialization failed")
        return

    try:
        messages = wx.GetAllMessage()
    except Exception as exc:
        emit_error("LISTEN_FAILED", f"fetch messages failed: {exc}", True)
        return

    if not messages:
        return

    chat_title = get_current_chat_title(wx)
    latest = messages[-1]
    text = extract_message_text(latest)
    if not text:
        return

    timestamp = int(time.time())
    msg_id = extract_msg_id(latest)
    key = msg_id or f"{text}:{timestamp}"
    if STATE.last_message_keys.get(chat_title) == key:
        return
    STATE.last_message_keys[chat_title] = key

    sender = extract_sender_name(latest) or chat_title
    payload = {
        "chat_id": chat_title,
        "chat_title": chat_title,
        "is_group": False,
        "sender_name": sender,
        "text": text,
        "timestamp": timestamp,
        "msg_id": msg_id,
    }
    send_with_ack("message.new", payload)


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
        emit_status("listening", "")
        return

    if msg_type == "listen.pause":
        STATE.listening = False
        emit_status("paused", "")
        return

    if msg_type == "listen.stop":
        STATE.listening = False
        emit_status("idle", "")
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
            "capabilities": ["listen", "write"],
            "supports_clipboard_restore": True,
        },
    )

    reader = threading.Thread(target=read_stdin, daemon=True)
    reader.start()

    last_poll = 0.0
    while True:
        try:
            message = COMMAND_QUEUE.get(timeout=0.1)
            handle_command(message)
        except queue.Empty:
            pass

        if STATE.listening and (time.time() - last_poll) >= STATE.poll_interval:
            poll_messages()
            last_poll = time.time()

        process_pending()


if __name__ == "__main__":
    main()
