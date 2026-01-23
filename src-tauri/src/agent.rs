use crate::ipc::IpcMessage;

pub fn parse_line(line: &str) -> Result<IpcMessage, serde_json::Error> {
    serde_json::from_str(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_inbound_line() {
        let line = r#"{"version":"1.0","type":"agent.ready","id":"1","timestamp":1,"payload":{}}"#;
        let msg = parse_line(line).unwrap();
        assert_eq!(msg.r#type, "agent.ready");
    }
}
