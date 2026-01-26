export type ListenTargetKind = "direct" | "group" | "unknown";

export type ListenTarget = {
  name: string;
  kind: ListenTargetKind;
};

export const MAX_LISTEN_TARGETS = 50;

export const normalizeListenTargets = (
  names: string[],
  kind: ListenTargetKind = "unknown",
): ListenTarget[] => {
  const seen = new Set<string>();
  const normalized: ListenTarget[] = [];
  for (const raw of names) {
    const name = raw.trim();
    if (!name || seen.has(name)) {
      continue;
    }
    seen.add(name);
    normalized.push({ name, kind });
  }
  return normalized;
};

export const normalizeListenTargetList = (
  targets: ListenTarget[],
): ListenTarget[] => {
  const seen = new Set<string>();
  const normalized: ListenTarget[] = [];
  for (const target of targets) {
    const name = target.name.trim();
    if (!name || seen.has(name)) {
      continue;
    }
    seen.add(name);
    normalized.push({ name, kind: target.kind });
  }
  return normalized;
};

export const mergeListenTargets = (
  base: ListenTarget[],
  additions: ListenTarget[],
): ListenTarget[] => normalizeListenTargetList([...base, ...additions]);
