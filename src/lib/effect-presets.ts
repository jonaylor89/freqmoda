export const effectPresets: Record<string, Record<string, string>> = {
  echo: {
    light: '0.6:0.3:1000:0.3',
    medium: '0.8:0.88:60:0.4',
    heavy: '0.8:0.9:1000:0.5',
  },
  chorus: {
    light: '0.5:0.9:50:0.4:0.25:2',
    medium: '0.7:0.9:50:0.4:0.25:2',
    heavy: '0.9:0.9:50:0.4:0.25:2',
  },
  flanger: {
    light: '0.5:0.75:2:0.25:2',
    medium: '0.7:0.75:3:0.25:2',
    heavy: '0.9:0.75:4:0.25:2',
  },
};

export function resolvePresets(params: Record<string, unknown>): Record<string, unknown> {
  const resolved = { ...params };
  for (const [effect, presets] of Object.entries(effectPresets)) {
    const value = resolved[effect];
    if (typeof value === 'string' && value in presets) {
      resolved[effect] = presets[value];
    }
  }
  return resolved;
}
