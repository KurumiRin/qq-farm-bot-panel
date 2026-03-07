import { useState, useEffect, useCallback, useRef, useLayoutEffect } from "react";
import { Package, ShoppingCart, Sprout, Apple, FlaskConical, RefreshCw } from "lucide-react";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";
import { useToast } from "../components/Toast";
import { useTauriEvent } from "../hooks/useTauriEvent";
import * as api from "../api";

interface BagItemView {
  id: number;
  count: number;
  name: string;
  category: string;
}

interface CurrencyView {
  id: number;
  count: number;
  name: string;
}

interface BagView {
  items: BagItemView[];
  currencies: CurrencyView[];
  seed_count: number;
  fruit_count: number;
  fertilizer_count: number;
  other_count: number;
  normal_fert_secs: number;
  organic_fert_secs: number;
}

const CATEGORY_LABELS: Record<string, { label: string; color: string; icon: typeof Package }> = {
  fruit: { label: "果实", color: "text-red-500", icon: Apple },
  seed: { label: "种子", color: "text-green-500", icon: Sprout },
  fertilizer: { label: "化肥", color: "text-purple-500", icon: FlaskConical },
  other: { label: "其他", color: "text-on-surface-muted", icon: Package },
};

const TABS = ["all", "fruit", "seed", "fertilizer", "other"] as const;
const TAB_LABELS: Record<string, string> = {
  all: "全部",
  fruit: "果实",
  seed: "种子",
  fertilizer: "化肥",
  other: "其他",
};

function TabBar({
  tabs,
  tab,
  setTab,
  items,
  bag,
}: {
  tabs: readonly string[];
  tab: string;
  setTab: (t: string) => void;
  items: BagItemView[];
  bag: BagView | null;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [indicator, setIndicator] = useState({ left: 0, width: 0, ready: false });
  const activeIndex = tabs.indexOf(tab);

  useLayoutEffect(() => {
    const container = containerRef.current;
    if (!container || activeIndex < 0) return;
    const buttons = container.querySelectorAll<HTMLElement>("button");
    const el = buttons[activeIndex];
    if (!el) return;
    setIndicator({ left: el.offsetLeft, width: el.offsetWidth, ready: true });
  }, [activeIndex]);

  return (
    <div ref={containerRef} className="relative flex gap-0.5 rounded-lg bg-surface-bright/70 p-0.5">
      {/* Sliding indicator */}
      <div
        className={`absolute top-0.5 bottom-0.5 rounded-md bg-surface shadow-sm pointer-events-none ${
          indicator.ready ? "transition-all duration-250 ease-in-out" : "opacity-0"
        }`}
        style={{ left: indicator.left, width: indicator.width }}
      />
      {tabs.map((t) => {
        const count =
          t === "all"
            ? items.length
            : t === "other"
              ? (bag?.other_count ?? 0)
              : items.filter((i) => i.category === t).length;
        return (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`relative z-1 flex-1 px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              tab === t
                ? "text-on-surface"
                : "text-on-surface-muted hover:text-on-surface"
            }`}
          >
            {TAB_LABELS[t]} {count > 0 && `(${count})`}
          </button>
        );
      })}
    </div>
  );
}

export default function InventoryPage() {
  const [bag, setBag] = useState<BagView | null>(null);
  const [loading, setLoading] = useState(false);
  const [selling, setSelling] = useState(false);
  const [tab, setTab] = useState<string>("all");
  const [tabKey, setTabKey] = useState(0);
  const { toast } = useToast();

  const handleSetTab = useCallback((newTab: string) => {
    setTab(newTab);
    setTabKey((k) => k + 1);
  }, []);

  const fetchBag = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getBag()) as BagView;
      setBag(res);
    } catch (e) {
      if (String(e) !== "Not connected") console.error("Failed to load bag:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBag();
  }, [fetchBag]);

  const handleDataChanged = useCallback(
    (scope: string) => {
      if (scope === "inventory") fetchBag();
    },
    [fetchBag]
  );
  useTauriEvent("data-changed", handleDataChanged);

  const handleStatusChanged = useCallback(
    (payload: { connection: string }) => {
      if (payload.connection === "LoggedIn") fetchBag();
      else if (payload.connection === "Disconnected") setBag(null);
    },
    [fetchBag]
  );
  useTauriEvent("status-changed", handleStatusChanged);

  const handleSellAll = async () => {
    setSelling(true);
    try {
      const result = await api.sellAllFruits();
      toast("success", result);
      await fetchBag();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      toast("error", msg);
    } finally {
      setSelling(false);
    }
  };

  const items = bag?.items ?? [];
  const filtered = tab === "all" ? items : items.filter((i) => i.category === tab);
  const hasFruits = items.some((i) => i.category === "fruit");

  return (
    <div className="space-y-4">
      <PageHeader
        title="仓库"
        tags={[
          { label: "金币", value: bag ? bag.currencies.find(c => c.id === 1 || c.id === 1001)?.count.toLocaleString() ?? "0" : "-", cls: "bg-amber-500/10 text-amber-700 dark:text-amber-400" },
          { label: "点券", value: bag ? bag.currencies.find(c => c.id === 1002)?.count.toLocaleString() ?? "0" : "-", cls: "bg-amber-500/10 text-amber-700 dark:text-amber-400" },
        ]}
        actions={<>
          <Button
            size="sm"
            variant="ghost"
            icon={<RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />}
            onClick={fetchBag}
            disabled={loading || selling}
          >
            刷新
          </Button>
          {hasFruits && (
            <Button
              size="sm"
              variant="danger"
              icon={<ShoppingCart className="size-3.5" />}
              onClick={handleSellAll}
              loading={selling}
            >
              卖出果实
            </Button>
          )}
        </>}
      />

      {/* Tabs */}
      <TabBar tabs={TABS} tab={tab} setTab={handleSetTab} items={items} bag={bag} />

      <div key={tabKey}>
      {filtered.length === 0 && !loading ? (
        <EmptyState
          icon={<Package className="size-10" />}
          title={tab === "all" ? "背包为空" : `没有${TAB_LABELS[tab]}`}
          description={tab === "all" ? "请先连接游戏服务器" : "当前分类下没有物品"}
        />
      ) : (
        <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8">
          {filtered.map((item, idx) => {
            const catInfo = CATEGORY_LABELS[item.category] ?? CATEGORY_LABELS.other;
            const isSeed = item.category === "seed";
            const seedId = isSeed ? item.id : item.category === "fruit" ? item.id - 20000 : 0;

            return (
              <div
                key={`${item.id}-${idx}`}
                className="animate-list-item rounded-lg border border-border bg-surface p-2 flex flex-col items-center gap-1"
                style={{ animationDelay: `${Math.min(idx * 15, 300)}ms` }}
              >
                <div className="size-10 flex items-center justify-center">
                  {seedId > 0 ? (
                    <img
                      src={`/seeds/${seedId}.png`}
                      alt=""
                      className="size-10 object-contain"
                      onError={(e) => {
                        (e.target as HTMLImageElement).style.display = "none";
                      }}
                    />
                  ) : (
                    <catInfo.icon className={`size-5 ${catInfo.color}`} />
                  )}
                </div>
                <span className="text-[11px] font-medium text-center truncate w-full">
                  {item.name}
                </span>
                <span className="text-xs font-bold text-primary-600">x{item.count}</span>
              </div>
            );
          })}
        </div>
      )}
      </div>
    </div>
  );
}
