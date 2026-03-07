import { useState, useEffect, useCallback, useRef } from "react";
import { Users, Scissors, Droplets, Leaf, Bug, RefreshCw, Zap, Radar } from "lucide-react";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";
import { useToast } from "../components/Toast";
import { useTauriEvent } from "../hooks/useTauriEvent";
import { useMinLoading } from "../hooks/useMinLoading";
import * as api from "../api";

interface FriendView {
  gid: number;
  name: string;
  level: number;
  avatar_url: string;
  steal_count: number;
  dry_count: number;
  weed_count: number;
  insect_count: number;
}

interface FriendsData {
  friends: FriendView[];
  application_count: number;
}

type Filter = "all" | "steal" | "dry" | "weed" | "insect" | "idle";

const filters: { key: Filter; label: string }[] = [
  { key: "all", label: "全部" },
  { key: "steal", label: "可偷" },
  { key: "dry", label: "旱" },
  { key: "weed", label: "草" },
  { key: "insect", label: "虫" },
  { key: "idle", label: "无操作" },
];

function matchFilter(f: FriendView, filter: Filter): boolean {
  switch (filter) {
    case "all": return true;
    case "steal": return f.steal_count > 0;
    case "dry": return f.dry_count > 0;
    case "weed": return f.weed_count > 0;
    case "insect": return f.insect_count > 0;
    case "idle": return f.steal_count === 0 && f.dry_count === 0 && f.weed_count === 0 && f.insect_count === 0;
  }
}

const VISIT_DEBOUNCE_MS = 2000;

export default function FriendsPage() {
  const [data, setData] = useState<FriendsData | null>(null);
  const [loading, setLoading] = useMinLoading();
  const [busy, setBusy] = useState<number | null>(null);
  const [filter, _setFilter] = useState<Filter>("all");
  const [filterKey, setFilterKey] = useState(0);
  const [visited, setVisited] = useState<Set<number>>(new Set());
  const orderRef = useRef<number[]>([]);
  const visitDebounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const autoRefreshRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const { toast } = useToast();

  const setFilter = (f: Filter) => {
    _setFilter(f);
    setFilterKey((k) => k + 1);
  };

  // Stabilize friend order: keep existing positions, append new friends at end
  const stabilize = (friends: FriendView[]): FriendView[] => {
    const order = orderRef.current;
    const newGids = new Set(friends.map((f) => f.gid));
    orderRef.current = order.filter((gid) => newGids.has(gid));
    for (const f of friends) {
      if (!orderRef.current.includes(f.gid)) orderRef.current.push(f.gid);
    }
    const posMap = new Map(orderRef.current.map((gid, i) => [gid, i]));
    return friends.slice().sort((a, b) => (posMap.get(a.gid) ?? 0) - (posMap.get(b.gid) ?? 0));
  };

  const fetchFriends = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getFriends()) as FriendsData;
      setData({ ...res, friends: stabilize(res.friends) });
    } catch (e) {
      if (String(e) !== "Not connected") console.error("Failed to load friends:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  // Schedule auto-refresh based on config interval (cached to avoid extra requests)
  const cachedIntervalRef = useRef<number>(295_000);
  const scheduleAutoRefresh = useCallback(() => {
    clearTimeout(autoRefreshRef.current);
    autoRefreshRef.current = setTimeout(() => {
      fetchFriends().then(scheduleAutoRefresh);
    }, cachedIntervalRef.current);
    // Refresh interval from config in background (non-blocking)
    api.getAutomationConfig().then((config) => {
      cachedIntervalRef.current = Math.max((config.intervals.friend_min - 5) * 1000, 2_000);
    }).catch(() => {});
  }, [fetchFriends]);

  // Initial fetch + start auto-refresh
  useEffect(() => {
    fetchFriends().then(scheduleAutoRefresh);
    return () => {
      clearTimeout(autoRefreshRef.current);
      clearTimeout(visitDebounceRef.current);
    };
  }, [fetchFriends, scheduleAutoRefresh]);

  // Debounced fetch after visit: resets timer on each new visit
  const scheduleDebouncedFetch = useCallback(() => {
    clearTimeout(visitDebounceRef.current);
    visitDebounceRef.current = setTimeout(() => {
      fetchFriends().then(() => {
        setVisited(new Set());
        scheduleAutoRefresh();
      });
    }, VISIT_DEBOUNCE_MS);
  }, [fetchFriends, scheduleAutoRefresh]);

  const handleStatusChanged = useCallback(
    (payload: { connection: string }) => {
      if (payload.connection === "LoggedIn") {
        fetchFriends().then(scheduleAutoRefresh);
      } else if (payload.connection === "Disconnected") {
        setData(null);
        setFilter("all");
        setVisited(new Set());
        clearTimeout(autoRefreshRef.current);
        clearTimeout(visitDebounceRef.current);
      }
    },
    [fetchFriends, scheduleAutoRefresh]
  );
  useTauriEvent("status-changed", handleStatusChanged);

  const handleVisit = async (friend: FriendView) => {
    setBusy(friend.gid);
    try {
      const result = await api.visitAndActFriend(friend.gid);
      toast("success", `${friend.name}: ${result}`);
      setVisited((s) => new Set(s).add(friend.gid));
      scheduleDebouncedFetch();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      toast("error", msg);
    } finally {
      setBusy(null);
    }
  };

  const handleVisitAll = async () => {
    const actionable = (data?.friends ?? []).filter(
      (f) => !visited.has(f.gid) &&
        (f.steal_count > 0 || f.dry_count > 0 || f.weed_count > 0 || f.insect_count > 0)
    );
    if (actionable.length === 0) return;
    for (let i = 0; i < actionable.length; i++) {
      const friend = actionable[i];
      setBusy(friend.gid);
      try {
        const result = await api.visitAndActFriend(friend.gid);
        toast("success", `${friend.name}: ${result}`);
        setVisited((s) => new Set(s).add(friend.gid));
      } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : String(e);
        toast("error", `${friend.name}: ${msg}`);
      }
      // Small delay between friends to avoid server rate limiting
      if (i < actionable.length - 1) {
        await new Promise((r) => setTimeout(r, 300));
      }
    }
    setBusy(null);
    scheduleDebouncedFetch();
  };

  const handleManualRefresh = () => {
    setVisited(new Set());
    clearTimeout(visitDebounceRef.current);
    fetchFriends().then(scheduleAutoRefresh);
  };

  const allFriends = data?.friends ?? [];
  const friends = allFriends.filter((f) => matchFilter(f, filter));

  const counts: Record<Filter, number> = {
    all: allFriends.length,
    steal: allFriends.filter((f) => f.steal_count > 0).length,
    dry: allFriends.filter((f) => f.dry_count > 0).length,
    weed: allFriends.filter((f) => f.weed_count > 0).length,
    insect: allFriends.filter((f) => f.insect_count > 0).length,
    idle: allFriends.filter((f) => f.steal_count === 0 && f.dry_count === 0 && f.weed_count === 0 && f.insect_count === 0).length,
  };

  return (
    <div className="space-y-4">
      <PageHeader
        title="好友"
        tags={data ? [
          { label: `${allFriends.length} 位好友种菜中` },
          { label: "待处理申请", value: data.application_count, cls: "bg-primary-500/10 text-primary-600 dark:text-primary-400", hidden: data.application_count === 0 },
        ] : [{ label: "加载中..." }]}
        actions={
          <div className="flex items-center gap-1.5">
            <Button
              size="sm"
              variant="ghost"
              icon={<RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />}
              onClick={handleManualRefresh}
              disabled={loading || busy !== null}
            >
              刷新
            </Button>
            {counts.all - counts.idle > 0 && (
              <Button
                size="sm"
                icon={<Radar className="size-3.5" />}
                onClick={handleVisitAll}
                loading={busy !== null}
              >
                一键巡逻
              </Button>
            )}
          </div>
        }
        tagActions={allFriends.length > 0 ? (
          <div className="flex items-center gap-1">
            {filters.map(({ key, label }) => {
              const count = counts[key];
              return (
                <button
                  key={key}
                  onClick={() => setFilter(key)}
                  className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium transition-colors ${
                    filter === key
                      ? "bg-primary-500 text-white"
                      : "bg-surface-bright text-on-surface-muted hover:text-on-surface"
                  }`}
                >
                  {label}
                  <span className={`text-[10px] ${filter === key ? "text-white/70" : "text-on-surface-muted/60"}`}>
                    {count}
                  </span>
                </button>
              );
            })}
          </div>
        ) : undefined}
      />

      {friends.length === 0 && !loading ? (
        <EmptyState
          icon={<Users className="size-10" />}
          title={data ? "无匹配好友" : "暂无好友数据"}
          description={data ? "切换筛选条件查看" : "请先连接游戏服务器"}
        />
      ) : (
        <div key={filterKey} className="space-y-1">
          {friends.map((friend, i) => {
            const hasAny =
              friend.steal_count > 0 ||
              friend.dry_count > 0 ||
              friend.weed_count > 0 ||
              friend.insect_count > 0;
            const isVisited = visited.has(friend.gid);

            return (
              <div
                key={friend.gid}
                className="animate-list-item flex items-center gap-3 rounded-lg border border-border bg-surface p-2.5"
                style={{ animationDelay: `${Math.min(i * 15, 200)}ms` }}
              >
                {/* Avatar */}
                <div className="size-9 rounded-full bg-surface-bright flex items-center justify-center shrink-0 overflow-hidden">
                  {friend.avatar_url ? (
                    <img src={friend.avatar_url} alt="" className="size-9 rounded-full object-cover" />
                  ) : (
                    <span className="text-xs font-bold text-on-surface-muted">{friend.name.charAt(0)}</span>
                  )}
                </div>

                {/* Info */}
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-1.5">
                    <span className="text-sm font-medium truncate">{friend.name}</span>
                    <span className="text-[11px] text-on-surface-muted shrink-0">Lv.{friend.level}</span>
                  </div>
                  <div className="flex items-center gap-1 mt-0.5 min-h-4.5">
                    {friend.steal_count > 0 && (
                      <span className="inline-flex items-center gap-px rounded px-1 py-px bg-orange-500/15 text-orange-600 dark:text-orange-400">
                        <Scissors className="size-2.5" />
                        <span className="text-[10px] font-medium">偷 {friend.steal_count}</span>
                      </span>
                    )}
                    {friend.dry_count > 0 && (
                      <span className="inline-flex items-center gap-px rounded px-1 py-px bg-blue-500/15 text-blue-600 dark:text-blue-400">
                        <Droplets className="size-2.5" />
                        <span className="text-[10px] font-medium">旱 {friend.dry_count}</span>
                      </span>
                    )}
                    {friend.weed_count > 0 && (
                      <span className="inline-flex items-center gap-px rounded px-1 py-px bg-emerald-500/15 text-emerald-600 dark:text-emerald-400">
                        <Leaf className="size-2.5" />
                        <span className="text-[10px] font-medium">草 {friend.weed_count}</span>
                      </span>
                    )}
                    {friend.insect_count > 0 && (
                      <span className="inline-flex items-center gap-px rounded px-1 py-px bg-red-500/15 text-red-600 dark:text-red-400">
                        <Bug className="size-2.5" />
                        <span className="text-[10px] font-medium">虫 {friend.insect_count}</span>
                      </span>
                    )}
                    {!hasAny && !isVisited && (
                      <span className="text-[10px] text-on-surface-muted/50">无需操作</span>
                    )}
                  </div>
                </div>

                {/* Action button */}
                {hasAny && !isVisited && (
                  <Button
                    size="sm"
                    variant="ghost"
                    icon={<Zap className="size-3.5" />}
                    onClick={() => handleVisit(friend)}
                    loading={busy === friend.gid}
                    disabled={busy !== null}
                  >
                    巡逻
                  </Button>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
