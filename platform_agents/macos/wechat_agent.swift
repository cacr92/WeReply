import Foundation

func send(_ json: String) {
    print(json)
    fflush(stdout)
}

send("""
{"version":"1.0","type":"agent.ready","id":"ready-1","timestamp":\(Int(Date().timeIntervalSince1970)),"payload":{"platform":"macos","agent_version":"0.1.0","capabilities":["listen","write"],"supports_clipboard_restore":true}}
""")

while let line = readLine() {
    if line.isEmpty { continue }
    send("""
{"version":"1.0","type":"agent.error","id":"err-1","timestamp":\(Int(Date().timeIntervalSince1970)),"payload":{"code":"PERMISSION_DENIED","message":"Accessibility permission required","recoverable":true}}
""")
}
