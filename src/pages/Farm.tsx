import { useState, useEffect, useCallback } from "react";
import { Sprout, Scissors, RefreshCw } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
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
  0: "Empty",
  1: "Seed",
  2: "Sprout",
  3: "Small",
  4: "Large",
  5: "Bloom",
  6: "Mature",
  7: "Dead",
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
          <h1 className="text-xl font-bold">My Farm</h1>
          <p className="text-sm text-on-surface-muted">
            {lands.length} lands | {matureCount} ready to harvest
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            size="sm"
            icon={<RefreshCw className="size-3.5" />}
            onClick={fetchLands}
            loading={loading}
          >
            Refresh
          </Button>
          {matureCount > 0 && (
            <Button
              size="sm"
              icon={<Scissors className="size-3.5" />}
              onClick={handleHarvestAll}
              loading={harvesting}
            >
              Harvest All ({matureCount})
            </Button>
          )}
        </div>
      </div>

      {lands.length === 0 && !loading ? (
        <EmptyState
          icon={<Sprout className="size-10" />}
          title="No land data"
          description="Connect to the game server to view your farm"
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
                      Land #{land.land_id ?? idx + 1}
                    </span>
                    <span
                      className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                        phaseColors[phase] ?? phaseColors[0]
                      }`}
                    >
                      {phaseLabels[phase] ?? "Unknown"}
                    </span>
                  </div>

                  {land.seed_name && (
                    <p className="text-sm font-medium truncate">
                      {land.seed_name}
                    </p>
                  )}

                  {phase === 6 && land.harvest_count != null && (
                    <p className="text-xs text-primary-600">
                      Yield: {land.harvest_count}
                    </p>
                  )}

                  {/* Status indicators */}
                  <div className="flex gap-1.5 flex-wrap">
                    {land.need_water && (
                      <span className="rounded bg-blue-100 px-1.5 py-0.5 text-xs text-blue-700">
                        Dry
                      </span>
                    )}
                    {land.need_weed_out && (
                      <span className="rounded bg-yellow-100 px-1.5 py-0.5 text-xs text-yellow-700">
                        Weeds
                      </span>
                    )}
                    {land.need_insecticide && (
                      <span className="rounded bg-red-100 px-1.5 py-0.5 text-xs text-red-700">
                        Bugs
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
