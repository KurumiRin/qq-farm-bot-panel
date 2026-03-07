import LEVEL_EXP from "../../shared-data/level_exp.json";

export function getLevelProgress(level: number, totalExp: number): { current: number; needed: number; ratio: number } {
  const idx = level - 1;
  if (idx < 0 || idx >= LEVEL_EXP.length) return { current: 0, needed: 1, ratio: 0 };
  const start = LEVEL_EXP[idx] ?? 0;
  const next = LEVEL_EXP[idx + 1] ?? start + 100000;
  const needed = next - start;
  const current = Math.max(0, totalExp - start);
  return { current, needed, ratio: Math.min(current / needed, 1) };
}
