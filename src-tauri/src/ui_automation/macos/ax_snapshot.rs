use serde_json::{json, Map, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct AxSnapshotRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxSnapshotInfo {
    pub role: Option<String>,
    pub title: Option<String>,
    pub value: Option<String>,
    pub frame: Option<AxSnapshotRect>,
    pub enabled: Option<bool>,
    pub focused: Option<bool>,
}

pub fn snapshot_tree<T: Clone>(
    root: T,
    depth: usize,
    info: &dyn Fn(&T) -> AxSnapshotInfo,
    children: &dyn Fn(&T) -> Vec<T>,
) -> Value {
    let details = info(&root);
    let mut node = Map::new();
    node.insert("role".to_string(), opt_string(details.role));
    node.insert("title".to_string(), opt_string(details.title));
    node.insert("value".to_string(), opt_string(details.value));
    node.insert(
        "frame".to_string(),
        details
            .frame
            .map(|frame| {
                json!({
                    "x": frame.x,
                    "y": frame.y,
                    "width": frame.width,
                    "height": frame.height,
                })
            })
            .unwrap_or(Value::Null),
    );
    node.insert(
        "enabled".to_string(),
        details.enabled.map(Value::Bool).unwrap_or(Value::Null),
    );
    node.insert(
        "focused".to_string(),
        details.focused.map(Value::Bool).unwrap_or(Value::Null),
    );
    let mut child_nodes = Vec::new();
    if depth > 0 {
        for child in children(&root) {
            child_nodes.push(snapshot_tree(child, depth - 1, info, children));
        }
    }
    node.insert("children".to_string(), Value::Array(child_nodes));
    Value::Object(node)
}

fn opt_string(value: Option<String>) -> Value {
    value.map(Value::String).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::{snapshot_tree, AxSnapshotInfo, AxSnapshotRect};

    #[derive(Clone)]
    struct TestNode {
        role: &'static str,
        title: &'static str,
        children: Vec<TestNode>,
    }

    impl TestNode {
        fn info(&self) -> AxSnapshotInfo {
            AxSnapshotInfo {
                role: Some(self.role.to_string()),
                title: Some(self.title.to_string()),
                value: None,
                frame: Some(AxSnapshotRect {
                    x: 1.0,
                    y: 2.0,
                    width: 3.0,
                    height: 4.0,
                }),
                enabled: Some(true),
                focused: Some(false),
            }
        }
    }

    fn children(node: &TestNode) -> Vec<TestNode> {
        node.children.clone()
    }

    #[test]
    fn snapshot_includes_children_and_metadata() {
        let root = TestNode {
            role: "AXWindow",
            title: "root",
            children: vec![TestNode {
                role: "AXGroup",
                title: "child",
                children: vec![],
            }],
        };
        let value = snapshot_tree(root, 2, &TestNode::info, &children);
        let obj = value.as_object().expect("root object");
        assert_eq!(obj.get("role").unwrap().as_str(), Some("AXWindow"));
        let children = obj.get("children").unwrap().as_array().unwrap();
        assert_eq!(children.len(), 1);
        let child = children[0].as_object().unwrap();
        assert_eq!(child.get("title").unwrap().as_str(), Some("child"));
        assert!(child.get("frame").unwrap().is_object());
    }

    #[test]
    fn snapshot_respects_depth_limit() {
        let root = TestNode {
            role: "AXWindow",
            title: "root",
            children: vec![TestNode {
                role: "AXGroup",
                title: "child",
                children: vec![TestNode {
                    role: "AXStaticText",
                    title: "leaf",
                    children: vec![],
                }],
            }],
        };
        let value = snapshot_tree(root, 1, &TestNode::info, &children);
        let obj = value.as_object().unwrap();
        let children = obj.get("children").unwrap().as_array().unwrap();
        let child = children[0].as_object().unwrap();
        let grand_children = child.get("children").unwrap().as_array().unwrap();
        assert!(grand_children.is_empty());
    }
}
