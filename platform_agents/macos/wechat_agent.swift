import AppKit
import ApplicationServices
import Foundation

private let ackTimeout: TimeInterval = 3
private let maxAckRetries = 3
private let defaultPollInterval: TimeInterval = 0.8

private struct PendingMessage {
    var envelope: [String: Any]
    var sentAt: Date
    var retries: Int
}

private final class AgentState {
    var listening = false
    var pollInterval = defaultPollInterval
    var lastMessageKeys: [String: String] = [:]
    var pending: [String: PendingMessage] = [:]
}

private let state = AgentState()

private func jsonString(_ object: Any) -> String? {
    guard JSONSerialization.isValidJSONObject(object) else { return nil }
    guard let data = try? JSONSerialization.data(withJSONObject: object) else { return nil }
    return String(data: data, encoding: .utf8)
}

private func sendEnvelope(type: String, payload: [String: Any], id: String? = nil, trackAck: Bool = true) {
    let envelope: [String: Any] = [
        "version": "1.0",
        "type": type,
        "id": id ?? UUID().uuidString,
        "timestamp": Int(Date().timeIntervalSince1970),
        "payload": payload,
    ]
    if let line = jsonString(envelope) {
        print(line)
        fflush(stdout)
    }
    if trackAck, let ackId = envelope["id"] as? String {
        state.pending[ackId] = PendingMessage(envelope: envelope, sentAt: Date(), retries: 0)
    }
}

private func sendAck(ackId: String, ok: Bool = true, error: String = "") {
    sendEnvelope(type: "event.ack", payload: ["ack_id": ackId, "ok": ok, "error": error], trackAck: false)
}

private func emitError(code: String, message: String, recoverable: Bool) {
    sendEnvelope(type: "agent.error", payload: [
        "code": code,
        "message": message,
        "recoverable": recoverable,
    ])
}

private func emitStatus(_ status: String, detail: String = "") {
    sendEnvelope(type: "agent.status", payload: ["state": status, "detail": detail])
}

private func checkAccessibility() -> Bool {
    let options = [kAXTrustedCheckOptionPrompt.takeRetainedValue() as String: false] as CFDictionary
    return AXIsProcessTrustedWithOptions(options)
}

private func frontmostWeChatApp() -> NSRunningApplication? {
    let bundleIds = ["com.tencent.xinWeChat", "com.tencent.WeChat"]
    for bundleId in bundleIds {
        if let app = NSRunningApplication.runningApplications(withBundleIdentifier: bundleId).first {
            return app
        }
    }
    return nil
}

private func frontmostWeChatWindow() -> AXUIElement? {
    guard let app = NSWorkspace.shared.frontmostApplication,
          let bundleId = app.bundleIdentifier,
          bundleId == "com.tencent.xinWeChat" || bundleId == "com.tencent.WeChat" else {
        return nil
    }
    let appElement = AXUIElementCreateApplication(app.processIdentifier)
    var value: CFTypeRef?
    let focusedResult = AXUIElementCopyAttributeValue(appElement, kAXFocusedWindowAttribute as CFString, &value)
    if focusedResult == .success, let window = value {
        return (window as! AXUIElement)
    }
    let windowsResult = AXUIElementCopyAttributeValue(appElement, kAXWindowsAttribute as CFString, &value)
    if windowsResult == .success, let windows = value as? [AXUIElement], let first = windows.first {
        return first
    }
    return nil
}

private func elementAttribute(_ element: AXUIElement, _ attribute: CFString) -> CFTypeRef? {
    var value: CFTypeRef?
    if AXUIElementCopyAttributeValue(element, attribute, &value) == .success {
        return value
    }
    return nil
}

private func collectStaticTexts(from element: AXUIElement, depth: Int, results: inout [String]) {
    guard depth > 0 else { return }
    if let role = elementAttribute(element, kAXRoleAttribute as CFString) as? String,
       role == kAXStaticTextRole as String,
       let value = elementAttribute(element, kAXValueAttribute as CFString) as? String {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty {
            results.append(trimmed)
        }
    }
    if let children = elementAttribute(element, kAXChildrenAttribute as CFString) as? [AXUIElement] {
        for child in children {
            collectStaticTexts(from: child, depth: depth - 1, results: &results)
        }
    }
}

private func findInputElement(in element: AXUIElement, depth: Int) -> AXUIElement? {
    guard depth > 0 else { return nil }
    if let role = elementAttribute(element, kAXRoleAttribute as CFString) as? String {
        if role == kAXTextAreaRole as String || role == kAXTextFieldRole as String {
            return element
        }
    }
    if let children = elementAttribute(element, kAXChildrenAttribute as CFString) as? [AXUIElement] {
        for child in children {
            if let found = findInputElement(in: child, depth: depth - 1) {
                return found
            }
        }
    }
    return nil
}

private func windowTitle(_ window: AXUIElement) -> String {
    if let title = elementAttribute(window, kAXTitleAttribute as CFString) as? String, !title.isEmpty {
        return title
    }
    return "未知会话"
}

private func pollMessages() {
    guard checkAccessibility() else {
        emitError(code: "PERMISSION_DENIED", message: "Accessibility permission required", recoverable: true)
        state.listening = false
        emitStatus("error", detail: "缺少辅助功能权限")
        return
    }
    guard let window = frontmostWeChatWindow() else { return }
    let title = windowTitle(window)
    var texts: [String] = []
    collectStaticTexts(from: window, depth: 6, results: &texts)
    guard let latest = texts.last else { return }
    let key = "\(latest):\(title)"
    if state.lastMessageKeys[title] == key { return }
    state.lastMessageKeys[title] = key

    let senderName: String
    if let colonIndex = latest.firstIndex(of: ":") {
        senderName = String(latest[..<colonIndex])
    } else {
        senderName = title
    }

    sendEnvelope(type: "message.new", payload: [
        "chat_id": title,
        "chat_title": title,
        "is_group": title.contains("群"),
        "sender_name": senderName,
        "text": latest,
        "timestamp": Int(Date().timeIntervalSince1970),
        "msg_id": nil,
    ])
}

private func pasteViaAppleScript() -> Bool {
    let script = "tell application \"System Events\" to keystroke \"v\" using {command down}"
    let appleScript = NSAppleScript(source: script)
    var error: NSDictionary?
    appleScript?.executeAndReturnError(&error)
    return error == nil
}

private func writeInput(chatId: String, text: String, restoreClipboard: Bool) {
    let _ = chatId
    guard checkAccessibility() else {
        sendEnvelope(type: "input.result", payload: ["ok": false, "error": "Accessibility permission missing"])
        return
    }
    guard let app = frontmostWeChatApp() else {
        sendEnvelope(type: "input.result", payload: ["ok": false, "error": "WeChat is not running"])
        return
    }
    app.activate(options: [.activateIgnoringOtherApps, .activateAllWindows])

    if let window = frontmostWeChatWindow(), let input = findInputElement(in: window, depth: 6) {
        AXUIElementSetAttributeValue(input, kAXFocusedAttribute as CFString, kCFBooleanTrue)
    }

    let pasteboard = NSPasteboard.general
    let previous = pasteboard.string(forType: .string)
    pasteboard.clearContents()
    pasteboard.setString(text, forType: .string)

    let ok = pasteViaAppleScript()
    sendEnvelope(type: "input.result", payload: ["ok": ok, "error": ok ? "" : "write failed"], trackAck: true)

    if restoreClipboard {
        pasteboard.clearContents()
        if let previous {
            pasteboard.setString(previous, forType: .string)
        }
    }
}

private func handleCommand(_ message: [String: Any]) {
    let msgType = message["type"] as? String ?? ""
    let msgId = message["id"] as? String ?? ""
    let payload = message["payload"] as? [String: Any] ?? [:]

    if msgType == "event.ack" {
        if let ackId = payload["ack_id"] as? String {
            state.pending.removeValue(forKey: ackId)
        }
        return
    }

    if !msgId.isEmpty {
        sendAck(ackId: msgId)
    }

    switch msgType {
    case "listen.start", "listen.resume":
        if let interval = payload["poll_interval_ms"] as? Double, interval >= 200 {
            state.pollInterval = max(interval / 1000.0, 0.2)
        }
        state.listening = true
        emitStatus("listening")
    case "listen.pause":
        state.listening = false
        emitStatus("paused")
    case "listen.stop":
        state.listening = false
        emitStatus("idle")
    case "input.write":
        let chatId = (payload["chat_id"] as? String ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
        let text = (payload["text"] as? String ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
        let restore = payload["restore_clipboard"] as? Bool ?? true
        if chatId.isEmpty || text.isEmpty {
            sendEnvelope(type: "input.result", payload: ["ok": false, "error": "chat_id 或内容为空"], trackAck: true)
        } else {
            writeInput(chatId: chatId, text: text, restoreClipboard: restore)
        }
    default:
        break
    }
}

private func readStdin() {
    while let line = readLine() {
        let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty { continue }
        guard let data = trimmed.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data),
              let dict = obj as? [String: Any] else {
            continue
        }
        handleCommand(dict)
    }
}

private func processPending() {
    let now = Date()
    for (id, pending) in state.pending {
        if now.timeIntervalSince(pending.sentAt) < ackTimeout { continue }
        if pending.retries >= maxAckRetries {
            state.pending.removeValue(forKey: id)
            continue
        }
        if let line = jsonString(pending.envelope) {
            print(line)
            fflush(stdout)
        }
        state.pending[id] = PendingMessage(envelope: pending.envelope, sentAt: now, retries: pending.retries + 1)
    }
}

sendEnvelope(type: "agent.ready", payload: [
    "platform": "macos",
    "agent_version": "0.1.0",
    "capabilities": ["listen", "write"],
    "supports_clipboard_restore": true,
])

DispatchQueue.global().async {
    readStdin()
}

let timer = DispatchSource.makeTimerSource(queue: DispatchQueue.global())
timer.schedule(deadline: .now(), repeating: .milliseconds(100))
timer.setEventHandler {
    if state.listening {
        pollMessages()
    }
    processPending()
}
timer.resume()

RunLoop.main.run()
