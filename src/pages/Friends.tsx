import { useState, useEffect, useCallback, useRef } from "react";
import { Users, Scissors, Droplets, Leaf, Bug, RefreshCw, Zap } from "lucide-react";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";
import { useToast } from "../components/Toast";
import { useTauriEvent } from "../hooks/useTauriEvent";
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

const CACHE_MIN_MS = 10_000; // Don't re-fetch within 10s

export default function FriendsPage() {
  const [data, setData] = useState<FriendsData | null>(null);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState<number | null>(null);
  const lastFetchRef = useRef(0);
  const { toast } = useToast();

  const fetchFriends = useCallback(async (force = false) => {
    if (!force && Date.now() - lastFetchRef.current < CACHE_MIN_MS) return;
    setLoading(true);
    try {
      const res = (await api.getFriends()) as FriendsData;
      setData(res);
      lastFetchRef.current = Date.now();
    } catch (e) {
      console.error("Failed to load friends:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchFriends(true);
  }, [fetchFriends]);

  const handleDataChanged = useCallback(
    (scope: string) => {
      if (scope === "friends") fetchFriends();
    },
    [fetchFriends]
  );
  useTauriEvent("data-changed", handleDataChanged);

  const handleStatusChanged = useCallback(
    (payload: { connection: string }) => {
      if (payload.connection === "LoggedIn") fetchFriends(true);
    },
    [fetchFriends]
  );
  useTauriEvent("status-changed", handleStatusChanged);

  const handleVisit = async (friend: FriendView) => {
    setBusy(friend.gid);
    try {
      const result = await api.visitAndActFriend(friend.gid);
      toast("success", `${friend.name}: ${result}`);
      await fetchFriends(true);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      toast("error", msg);
    } finally {
      setBusy(null);
    }
  };

  const friends = data?.friends ?? [];
  const hasAction = friends.filter(
    (f) => f.steal_count > 0 || f.dry_count > 0 || f.weed_count > 0 || f.insect_count > 0
  );

  return (
    <div className="space-y-4">
      <PageHeader
        title="好友"
        tags={data ? [
          { label: "好友", value: friends.length },
          { label: "可操作", value: hasAction.length, cls: "bg-green-500/10 text-green-700 dark:text-green-400", hidden: hasAction.length === 0 },
          { label: "待处理申请", value: data.application_count, cls: "bg-primary-500/10 text-primary-600 dark:text-primary-400", hidden: data.application_count === 0 },
        ] : [{ label: "加载中..." }]}
        actions={
          <Button
            size="sm"
            variant="ghost"
            icon={<RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />}
            onClick={() => fetchFriends(true)}
            disabled={loading || busy !== null}
          >
            刷新
          </Button>
        }
      />

      {friends.length === 0 && !loading ? (
        <EmptyState
          icon={<Users className="size-10" />}
          title="暂无好友数据"
          description="请先连接游戏服务器"
        />
      ) : (
        <div className="space-y-1">
          {friends.map((friend) => {
            const hasAny =
              friend.steal_count > 0 ||
              friend.dry_count > 0 ||
              friend.weed_count > 0 ||
              friend.insect_count > 0;

            return (
              <div
                key={friend.gid}
                className="flex items-center gap-3 rounded-lg border border-border bg-surface p-2.5"
              >
                {/* Avatar */}
                <div className="size-9 rounded-full bg-surface-bright flex items-center justify-center shrink-0 overflow-hidden">
                  {friend.avatar_url ? (
                    <img
                      src={friend.avatar_url}
                      alt=""
                      className="size-9 rounded-full object-cover"
                    />
                  ) : (
                    <span className="text-xs font-bold text-on-surface-muted">
                      {friend.name.charAt(0)}
                    </span>
                  )}
                </div>

                {/* Info */}
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-1.5">
                    <span className="text-sm font-medium truncate">{friend.name}</span>
                    <span className="text-[11px] text-on-surface-muted shrink-0">
                      Lv.{friend.level}
                    </span>
                  </div>
                  {/* Status tags */}
                  <div className="flex items-center gap-1 mt-0.5">
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
                    {!hasAny && (
                      <span className="text-[10px] text-on-surface-muted/50">无需操作</span>
                    )}
                  </div>
                </div>

                {/* Action button */}
                {hasAny && (
                  <Button
                    size="sm"
                    variant="ghost"
                    icon={<Zap className="size-3.5" />}
                    onClick={() => handleVisit(friend)}
                    loading={busy === friend.gid}
                    disabled={busy !== null}
                  >
                    执行
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
