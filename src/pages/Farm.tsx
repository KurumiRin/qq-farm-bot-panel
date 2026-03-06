import { useState, useEffect, useCallback } from "react";
import { Sprout, Scissors } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { useTauriEvent } from "../hooks/useTauriEvent";
import * as api from "../api";

interface LandInfo {
  land_id?: number;
  seed_id?: number;
  seed_name?: string;
  phase?: number;
  harvest_count?: number;
  need_water?: boolean;
  need_weed_out?: boolean;
  need_insecticide?: boolean;
  unlocked?: boolean;
}

const phaseLabels: Record<number, string> = {
  0: "空地",
  1: "种子",
  2: "发芽",
  3: "小叶",
  4: "大叶",
  5: "开花",
  6: "成熟",
  7: "枯死",
};

const phaseColors: Record<number, string> = {
  0: "bg-gray-100 text-gray-600",
  1: "bg-amber-100 text-amber-700",
  2: "bg-lime-100 text-lime-700",
  3: "bg-green-100 text-green-700",
  4: "bg-green-200 text-green-800",
  5: "bg-pink-100 text-pink-700",
  6: "bg-primary-100 text-primary-700",
  7: "bg-red-100 text-red-700",
};

export default function FarmPage() {
  const [lands, setLands] = useState<LandInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [harvesting, setHarvesting] = useState(false);

  const fetchLands = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getAllLands()) as { lands?: LandInfo[] };
      setLands(res?.lands ?? []);
    } catch (e) {
      console.error("Failed to load lands:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchLands();
  }, [fetchLands]);

  // Auto-refresh when automation completes a farm cycle
  const handleDataChanged = useCallback(
    (scope: string) => {
      if (scope === "farm") fetchLands();
    },
    [fetchLands]
  );
  useTauriEvent("data-changed", handleDataChanged);

  const handleHarvestAll = async () => {
    const matureIds = lands
      .filter((l) => l.phase === 6)
      .map((l) => l.land_id!)
      .filter(Boolean);
    if (matureIds.length === 0) return;

    setHarvesting(true);
    try {
      await api.harvest(matureIds);
      await fetchLands();
    } catch (e) {
      console.error("Harvest failed:", e);
    } finally {
      setHarvesting(false);
    }
  };

  const matureCount = lands.filter((l) => l.phase === 6).length;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">我的农场</h1>
          <p className="text-sm text-on-surface-muted">
            {lands.length} 块土地 | {matureCount} 块可收获
          </p>
        </div>
        {matureCount > 0 && (
          <Button
            size="sm"
            icon={<Scissors className="size-3.5" />}
            onClick={handleHarvestAll}
            loading={harvesting}
          >
            全部收获 ({matureCount})
          </Button>
        )}
      </div>

      {lands.length === 0 && !loading ? (
        <EmptyState
          icon={<Sprout className="size-10" />}
          title="暂无土地数据"
          description="请先连接游戏服务器"
        />
      ) : (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
          {lands.map((land, idx) => {
            const phase = land.phase ?? 0;
            return (
              <Card key={land.land_id ?? idx}>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-on-surface-muted">
                      土地 #{land.land_id ?? idx + 1}
                    </span>
                    <span
                      className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                        phaseColors[phase] ?? phaseColors[0]
                      }`}
                    >
                      {phaseLabels[phase] ?? "未知"}
                    </span>
                  </div>

                  {land.seed_name && (
                    <p className="text-sm font-medium truncate">
                      {land.seed_name}
                    </p>
                  )}

                  {phase === 6 && land.harvest_count != null && (
                    <p className="text-xs text-primary-600">
                      产量: {land.harvest_count}
                    </p>
                  )}

                  <div className="flex gap-1.5 flex-wrap">
                    {land.need_water && (
                      <span className="rounded bg-blue-100 px-1.5 py-0.5 text-xs text-blue-700">
                        缺水
                      </span>
                    )}
                    {land.need_weed_out && (
                      <span className="rounded bg-yellow-100 px-1.5 py-0.5 text-xs text-yellow-700">
                        有草
                      </span>
                    )}
                    {land.need_insecticide && (
                      <span className="rounded bg-red-100 px-1.5 py-0.5 text-xs text-red-700">
                        有虫
                      </span>
                    )}
                  </div>
                </div>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}
