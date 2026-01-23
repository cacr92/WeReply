import json
import sys
import time


def send(msg):
    sys.stdout.write(json.dumps(msg, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def main():
    send({
        "version": "1.0",
        "type": "agent.ready",
        "id": "ready-1",
        "timestamp": int(time.time()),
        "payload": {
            "platform": "windows",
            "agent_version": "0.1.0",
            "capabilities": ["listen", "write"],
            "supports_clipboard_restore": True,
        },
    })

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        if msg.get("type") == "input.write":
            send({
                "version": "1.0",
                "type": "input.result",
                "id": "result-1",
                "timestamp": int(time.time()),
                "payload": {"ok": True, "error": ""},
            })


if __name__ == "__main__":
    main()
