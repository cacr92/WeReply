export const DEFAULT_MODELS = ["deepseek-chat", "deepseek-reasoner"];

export const normalizeModels = (models: string[]): string[] => {
  const normalized = DEFAULT_MODELS.filter((model) => models.includes(model));
  return normalized.length > 0 ? normalized : [...DEFAULT_MODELS];
};

export const resolveModelSelection = (
  models: string[],
  selected: string,
): { selected: string; changed: boolean } => {
  if (models.includes(selected)) {
    return { selected, changed: false };
  }
  const next = models[0] ?? DEFAULT_MODELS[0];
  return { selected: next, changed: true };
};
