import { useState, useEffect, useCallback } from "react";
import { Users, Scissors, Droplets, Leaf, Bug, RefreshCw, Zap } from "lucide-react";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
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

export default function FriendsPage() {
  const [data, setData] = useState<FriendsData | null>(null);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState<number | null>(null);
  const { toast } = useToast();

  const fetchFriends = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getFriends()) as FriendsData;
      setData(res);
    } catch (e) {
      console.error("Failed to load friends:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchFriends();
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
      if (payload.connection === "LoggedIn") fetchFriends();
    },
    [fetchFriends]
  );
  useTauriEvent("status-changed", handleStatusChanged);

  const handleVisit = async (friend: FriendView) => {
    setBusy(friend.gid);
    try {
      const result = await api.visitAndActFriend(friend.gid);
      toast("success", `${friend.name}: ${result}`);
      await fetchFriends();
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
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">好友</h1>
          <p className="text-sm text-on-surface-muted mt-0.5">
            {data
              ? `${friends.length} 位好友 · ${hasAction.length} 位有可操作`
              : "加载中..."}
            {data && data.application_count > 0 && (
              <span className="ml-1 text-primary-500">
                · {data.application_count} 待处理申请
              </span>
            )}
          </p>
        </div>
        <Button
          size="sm"
          variant="ghost"
          icon={<RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />}
          onClick={fetchFriends}
          disabled={loading || busy !== null}
        >
          刷新
        </Button>
      </div>

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
