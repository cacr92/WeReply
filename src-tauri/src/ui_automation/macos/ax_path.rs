#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxPathStep {
    pub roles: &'static [&'static str],
    pub title_contains: Option<&'static str>,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxNodeInfo {
    pub role: Option<String>,
    pub title: Option<String>,
}

pub const fn step(
    roles: &'static [&'static str],
    index: usize,
    title_contains: Option<&'static str>,
) -> AxPathStep {
    AxPathStep {
        roles,
        title_contains,
        index,
    }
}

pub fn resolve_path<T: Clone>(
    root: T,
    steps: &[AxPathStep],
    info: impl Fn(&T) -> AxNodeInfo,
    children: impl Fn(&T) -> Vec<T>,
) -> Option<T> {
    let mut current = root;
    for step in steps {
        let mut matches = Vec::new();
        for child in children(&current) {
            if matches_step(&info(&child), step) {
                matches.push(child);
            }
        }
        if step.index >= matches.len() {
            return None;
        }
        current = matches[step.index].clone();
    }
    Some(current)
}

fn matches_step(info: &AxNodeInfo, step: &AxPathStep) -> bool {
    let Some(role) = info.role.as_deref() else {
        return false;
    };
    if !step.roles.iter().any(|candidate| candidate == &role) {
        return false;
    }
    if let Some(substr) = step.title_contains {
        let title = info.title.as_deref().unwrap_or("");
        if !title.contains(substr) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{resolve_path, step, AxNodeInfo, AxPathStep};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestNode {
        role: &'static str,
        title: &'static str,
        children: Vec<TestNode>,
    }

    impl TestNode {
        fn info(&self) -> AxNodeInfo {
            AxNodeInfo {
                role: Some(self.role.to_string()),
                title: Some(self.title.to_string()),
            }
        }
    }

    fn children(node: &TestNode) -> Vec<TestNode> {
        node.children.clone()
    }

    #[test]
    fn resolves_nth_match_by_role() {
        let root = TestNode {
            role: "AXWindow",
            title: "",
            children: vec![
                TestNode {
                    role: "AXGroup",
                    title: "left",
                    children: vec![],
                },
                TestNode {
                    role: "AXGroup",
                    title: "right",
                    children: vec![],
                },
            ],
        };
        let steps: &[AxPathStep] = &[step(&["AXGroup"], 1, None)];
        let found = resolve_path(root.clone(), steps, TestNode::info, children).unwrap();
        assert_eq!(found.title, "right");
    }

    #[test]
    fn resolves_with_title_contains() {
        let root = TestNode {
            role: "AXWindow",
            title: "",
            children: vec![
                TestNode {
                    role: "AXGroup",
                    title: "ChatList",
                    children: vec![],
                },
                TestNode {
                    role: "AXGroup",
                    title: "MessagesPane",
                    children: vec![],
                },
            ],
        };
        let steps: &[AxPathStep] = &[step(&["AXGroup"], 0, Some("Messages"))];
        let found = resolve_path(root.clone(), steps, TestNode::info, children).unwrap();
        assert_eq!(found.title, "MessagesPane");
    }

    #[test]
    fn returns_none_when_out_of_range() {
        let root = TestNode {
            role: "AXWindow",
            title: "",
            children: vec![TestNode {
                role: "AXGroup",
                title: "only",
                children: vec![],
            }],
        };
        let steps: &[AxPathStep] = &[step(&["AXGroup"], 2, None)];
        let found = resolve_path(root.clone(), steps, TestNode::info, children);
        assert!(found.is_none());
    }
}
