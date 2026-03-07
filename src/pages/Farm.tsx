import { useState, useEffect, useCallback, useRef } from "react";
import { Sprout, Scissors, Droplets, Bug, Leaf, Lock, Trash2, RefreshCw, Shovel, Coins, Star } from "lucide-react";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";
import { useToast } from "../components/Toast";
import { useTauriEvent } from "../hooks/useTauriEvent";
import * as api from "../api";

// --- Types ---

interface LandView {
  id: number;
  unlocked: boolean;
  level: number;
  max_level: number;
  status: "locked" | "empty" | "growing" | "mature" | "dead";
  seed_id: number;
  seed_name: string;
  phase: number;
  phase_name: string;
  mature_in_sec: number;
  total_grow_sec: number;
  fruit_num: number;
  need_water: boolean;
  need_weed: boolean;
  need_insect: boolean;
  est_gold: number;
  est_exp: number;
  seasons: number;
}

interface FarmSummary {
  total: number;
  unlocked: number;
  mature: number;
  growing: number;
  empty: number;
  dead: number;
  need_water: number;
  need_weed: number;
  need_insect: number;
}

interface FarmView {
  lands: LandView[];
  summary: FarmSummary;
}

// --- Helpers ---

const SOIL: Record<number, { card: string; bar: string; label: string }> = {
  1: { card: "bg-amber-50 dark:bg-amber-950/40 border-amber-300/60 dark:border-amber-700/50", bar: "bg-amber-400", label: "黄土" },
  2: { card: "bg-red-50 dark:bg-red-950/40 border-red-300/60 dark:border-red-700/50", bar: "bg-red-400", label: "红土" },
  3: { card: "bg-stone-100 dark:bg-stone-900/50 border-stone-400/50 dark:border-stone-600/50", bar: "bg-stone-500", label: "黑土" },
  4: { card: "bg-yellow-50 dark:bg-yellow-950/40 border-yellow-300/70 dark:border-yellow-600/50", bar: "bg-yellow-400", label: "金土" },
};

function formatTime(sec: number): string {
  if (sec <= 0) return "";
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${m}:${String(s).padStart(2, "0")}`;
}

function formatNum(n: number): string {
  if (Math.abs(n) >= 10000) return `${(n / 10000).toFixed(1)}w`;
  if (Math.abs(n) >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return String(n);
}

function getProgress(land: LandView): number {
  if (land.status === "mature") return 1;
  if (land.status === "dead" || land.status === "empty" || !land.total_grow_sec) return 0;
  const elapsed = land.total_grow_sec - land.mature_in_sec;
  return Math.min(1, Math.max(0, elapsed / land.total_grow_sec));
}

// --- Components ---

function LandCard({ land }: { land: LandView }) {
  const soil = SOIL[land.level];
  const isMature = land.status === "mature";
  const isDead = land.status === "dead";
  const isEmpty = land.status === "empty" || !land.unlocked;
  const isGrowing = land.status === "growing";
  const progress = getProgress(land);

  const cardBg = !land.unlocked
    ? "bg-surface border-border opacity-35"
    : soil?.card ?? "bg-surface border-border";

  return (
    <div className={`rounded-lg border p-1.5 flex flex-col gap-0.5 ${cardBg}`}>
      {/* Row 1: id + soil label + status + needs */}
      <div className="flex items-center gap-1 text-[10px] leading-none min-h-3.5">
        <span className="text-on-surface-muted/60">#{land.id}</span>
        {soil && land.unlocked && (
          <span className="text-on-surface-muted/80 font-medium">{soil.label}</span>
        )}
        {isMature && (
          <span className="rounded px-1 py-0.5 font-semibold bg-green-500/20 text-green-700 dark:text-green-400">熟</span>
        )}
        {isDead && (
          <span className="rounded px-1 py-0.5 font-semibold bg-red-500/20 text-red-600">枯</span>
        )}
        <div className="flex gap-0.5 ml-auto">
          {land.need_water && (
            <span className="inline-flex items-center gap-0.5 rounded px-1 py-px bg-blue-500/15 text-blue-600 dark:text-blue-400">
              <Droplets className="size-2.5" /><span className="text-[9px] font-medium">旱</span>
            </span>
          )}
          {land.need_weed && (
            <span className="inline-flex items-center gap-0.5 rounded px-1 py-px bg-emerald-500/15 text-emerald-600 dark:text-emerald-400">
              <Leaf className="size-2.5" /><span className="text-[9px] font-medium">草</span>
            </span>
          )}
          {land.need_insect && (
            <span className="inline-flex items-center gap-0.5 rounded px-1 py-px bg-red-500/15 text-red-600 dark:text-red-400">
              <Bug className="size-2.5" /><span className="text-[9px] font-medium">虫</span>
            </span>
          )}
        </div>
      </div>

      {/* Row 2: image + info */}
      <div className="flex items-center gap-1.5 min-h-10">
        <div className="shrink-0 size-10 flex items-center justify-center">
          {!land.unlocked ? (
            <Lock className="size-4 text-on-surface-muted/20" />
          ) : !isEmpty && land.seed_id > 0 ? (
            <img src={`/seeds/${land.seed_id}.png`} alt={land.seed_name} className="size-10 object-contain" />
          ) : (
            <Sprout className="size-4 text-on-surface-muted/20" />
          )}
        </div>
        <div className="flex-1 min-w-0 space-y-0.5">
          <span className="text-[11px] font-medium truncate block leading-tight">
            {!land.unlocked ? "未开垦" : isEmpty ? "空地" : land.seed_name || "未知"}
          </span>
          {isGrowing && (
            <span className="text-[10px] text-on-surface-muted leading-tight block">{land.phase_name}</span>
          )}
          {isMature && land.fruit_num > 0 && (
            <span className="text-[10px] text-green-600 dark:text-green-400 leading-tight block">×{land.fruit_num}</span>
          )}
        </div>
      </div>

      {/* Row 3: progress bar + time */}
      {isGrowing && land.total_grow_sec > 0 ? (
        <div className="space-y-0.5">
          <div className="h-1 rounded-full bg-black/5 dark:bg-white/10 overflow-hidden">
            <div
              className={`h-full rounded-full transition-all duration-1000 ${progress >= 1 ? "bg-green-500" : "bg-blue-500"}`}
              style={{ width: `${(progress * 100).toFixed(1)}%` }}
            />
          </div>
          {land.mature_in_sec > 0 && (
            <span className="text-[9px] text-on-surface-muted tabular-nums block text-right">{formatTime(land.mature_in_sec)}</span>
          )}
        </div>
      ) : isMature ? (
        <div className="h-1 rounded-full bg-green-500" />
      ) : null}

      {/* Row 4: earnings estimate */}
      {!isEmpty && land.seed_id > 0 && (land.est_gold !== 0 || land.est_exp !== 0) && (
        <div className="flex items-center gap-1.5 text-[9px] text-on-surface-muted/70 leading-none">
          {land.est_gold !== 0 && (
            <span className="inline-flex items-center gap-px">
              <Coins className="size-2.5 text-amber-500/70" />
              {land.est_gold > 0 ? "+" : ""}{formatNum(land.est_gold)}
            </span>
          )}
          {land.est_exp > 0 && (
            <span className="inline-flex items-center gap-px">
              <Star className="size-2.5 text-blue-500/70" />
              +{formatNum(land.est_exp)}
            </span>
          )}
          {land.seasons >= 2 && (
            <span className="text-purple-500/70 font-medium">2季</span>
          )}
        </div>
      )}
    </div>
  );
}

// --- Main Page ---

export default function FarmPage() {
  const [farm, setFarm] = useState<FarmView | null>(null);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const { toast } = useToast();
  const timerRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const fetchLands = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getAllLands()) as FarmView;
      setFarm(res);
    } catch (e) {
      if (String(e) !== "Not connected") console.error("Failed to load lands:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchLands();
  }, [fetchLands]);

  useEffect(() => {
    timerRef.current = setInterval(() => {
      setFarm((prev) => {
        if (!prev) return prev;
        const hasCountdown = prev.lands.some((l) => l.mature_in_sec > 0);
        if (!hasCountdown) return prev;
        return {
          ...prev,
          lands: prev.lands.map((l) =>
            l.mature_in_sec > 0
              ? {
                  ...l,
                  mature_in_sec: l.mature_in_sec - 1,
                  ...(l.mature_in_sec <= 1
                    ? { status: "mature" as const, phase_name: "成熟" }
                    : {}),
                }
              : l
          ),
        };
      });
    }, 1000);
    return () => clearInterval(timerRef.current);
  }, []);

  const handleDataChanged = useCallback(
    (scope: string) => {
      if (scope === "farm") fetchLands();
    },
    [fetchLands]
  );
  useTauriEvent("data-changed", handleDataChanged);

  const handleStatusChanged = useCallback(
    (payload: { connection: string }) => {
      if (payload.connection === "LoggedIn") fetchLands();
      else if (payload.connection === "Disconnected") setFarm(null);
    },
    [fetchLands]
  );
  useTauriEvent("status-changed", handleStatusChanged);

  const runAction = async (key: string, fn: () => Promise<unknown>, successMsg?: string) => {
    setBusy(key);
    try {
      const result = await fn();
      await fetchLands();
      toast("success", typeof result === "string" ? result : successMsg ?? "操作成功");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      toast("error", msg);
    } finally {
      setBusy(null);
    }
  };

  const summary = farm?.summary;
  const lands = farm?.lands ?? [];

  const waterIds = lands.filter((l) => l.need_water).map((l) => l.id);
  const weedIds = lands.filter((l) => l.need_weed).map((l) => l.id);
  const insectIds = lands.filter((l) => l.need_insect).map((l) => l.id);
  const matureIds = lands.filter((l) => l.status === "mature").map((l) => l.id);
  const deadIds = lands.filter((l) => l.status === "dead").map((l) => l.id);
  const emptyIds = lands.filter((l) => l.unlocked && l.status === "empty").map((l) => l.id);

  const totalGold = lands.reduce((s, l) => s + (l.status !== "dead" && l.status !== "empty" ? l.est_gold : 0), 0);
  const totalExp = lands.reduce((s, l) => s + (l.status !== "dead" && l.status !== "empty" ? l.est_exp : 0), 0);

  return (
    <div className="space-y-4">
      <PageHeader
        title="我的农场"
        tags={[
          ...(() => {
            const soilCounts: Record<number, number> = {};
            for (const l of lands) {
              if (l.unlocked) soilCounts[l.level] = (soilCounts[l.level] ?? 0) + 1;
            }
            const SOIL_TAG: Record<number, { label: string; cls: string }> = {
              1: { label: "黄土", cls: "bg-amber-500/10 text-amber-700 dark:text-amber-400" },
              2: { label: "红土", cls: "bg-red-500/10 text-red-700 dark:text-red-400" },
              3: { label: "黑土", cls: "bg-stone-500/10 text-stone-700 dark:text-stone-300" },
              4: { label: "金土", cls: "bg-yellow-500/10 text-yellow-700 dark:text-yellow-400" },
            };
            return Object.entries(soilCounts)
              .sort(([a], [b]) => Number(a) - Number(b))
              .map(([lvl, count]) => {
                const info = SOIL_TAG[Number(lvl)];
                return info ? { label: info.label, value: count, cls: info.cls } : null;
              })
              .filter(Boolean) as { label: string; value: number; cls: string }[];
          })(),
          { label: "生长中", value: summary?.growing ?? "-", cls: "bg-green-500/10 text-green-700 dark:text-green-400" },
          { label: "可收获", value: summary?.mature ?? 0, cls: "bg-amber-500/10 text-amber-700 dark:text-amber-400" },
          { label: "", value: formatNum(totalGold), icon: <Coins className="size-3" />, cls: "bg-yellow-500/10 text-yellow-700 dark:text-yellow-400", hidden: totalGold === 0 },
          { label: "", value: formatNum(totalExp), icon: <Star className="size-3" />, cls: "bg-blue-500/10 text-blue-700 dark:text-blue-400", hidden: totalExp <= 0 },
        ]}
        actions={<>
          <Button
            size="sm"
            variant="ghost"
            icon={<RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />}
            onClick={fetchLands}
            disabled={loading || !!busy}
          >
            刷新
          </Button>
          {matureIds.length > 0 && (
            <Button
              size="sm"
              icon={<Scissors className="size-3.5" />}
              onClick={() => runAction("harvest", () => api.harvest(matureIds), `收获 ${matureIds.length} 块地`)}
              loading={busy === "harvest"}
              disabled={!!busy}
            >
              收获 ({matureIds.length})
            </Button>
          )}
          {waterIds.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              icon={<Droplets className="size-3.5 text-blue-500" />}
              onClick={() => runAction("water", () => api.waterLands(waterIds), `浇水 ${waterIds.length} 块地`)}
              loading={busy === "water"}
              disabled={!!busy}
            >
              浇水 ({waterIds.length})
            </Button>
          )}
          {weedIds.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              icon={<Leaf className="size-3.5 text-emerald-500" />}
              onClick={() => runAction("weed", () => api.weedOutLands(weedIds), `除草 ${weedIds.length} 块地`)}
              loading={busy === "weed"}
              disabled={!!busy}
            >
              除草 ({weedIds.length})
            </Button>
          )}
          {insectIds.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              icon={<Bug className="size-3.5 text-red-500" />}
              onClick={() => runAction("insect", () => api.insecticideLands(insectIds), `除虫 ${insectIds.length} 块地`)}
              loading={busy === "insect"}
              disabled={!!busy}
            >
              除虫 ({insectIds.length})
            </Button>
          )}
          {deadIds.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              icon={<Trash2 className="size-3.5 text-on-surface-muted" />}
              onClick={() => runAction("remove", () => api.removeDeadPlants(deadIds), `铲除 ${deadIds.length} 块地`)}
              loading={busy === "remove"}
              disabled={!!busy}
            >
              铲除 ({deadIds.length})
            </Button>
          )}
          {emptyIds.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              icon={<Shovel className="size-3.5 text-amber-600" />}
              onClick={() => runAction("plant", () => api.autoPlantEmpty(emptyIds))}
              loading={busy === "plant"}
              disabled={!!busy}
            >
              种植 ({emptyIds.length})
            </Button>
          )}
        </>}
      />

      {lands.length === 0 && !loading ? (
        <EmptyState
          icon={<Sprout className="size-10" />}
          title="暂无土地数据"
          description="请先连接游戏服务器"
        />
      ) : (
        <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8">
          {lands.map((land, i) => (
            <div key={land.id} className="animate-list-item" style={{ animationDelay: `${Math.min(i * 20, 200)}ms` }}>
              <LandCard land={land} />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
