use crate::types::ListenTarget;
use anyhow::Result;
use std::collections::HashSet;

#[cfg(test)]
use crate::types::ChatKind;

pub const MAX_LISTEN_TARGETS: usize = 50;

pub fn normalize_listen_targets(targets: Vec<ListenTarget>, max: usize) -> Result<Vec<ListenTarget>> {
    if max == 0 {
        return Ok(Vec::new());
    }
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for mut target in targets {
        let trimmed = target.name.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.contains(trimmed) {
            continue;
        }
        target.name = trimmed.to_string();
        seen.insert(target.name.clone());
        normalized.push(target);
        if normalized.len() >= max {
            break;
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_dedupes_targets() {
        let input = vec![
            ListenTarget {
                name: "  Team A ".into(),
                kind: ChatKind::Unknown,
            },
            ListenTarget {
                name: "Team A".into(),
                kind: ChatKind::Unknown,
            },
            ListenTarget {
                name: "".into(),
                kind: ChatKind::Unknown,
            },
        ];
        let out = normalize_listen_targets(input, 50).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "Team A");
    }
}
