import { useState, useEffect, useCallback } from "react";
import { Users, RefreshCw } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import * as api from "../api";

interface FriendInfo {
  gid?: number;
  name?: string;
  level?: number;
  avatar_url?: string;
  can_steal?: boolean;
  can_water?: boolean;
}

export default function FriendsPage() {
  const [friends, setFriends] = useState<FriendInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchFriends = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getFriends()) as { friends?: FriendInfo[] };
      setFriends(res?.friends ?? (Array.isArray(res) ? res : []));
    } catch (e) {
      console.error("Failed to load friends:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchFriends();
  }, [fetchFriends]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">好友</h1>
          <p className="text-sm text-on-surface-muted">
            共 {friends.length} 位好友
          </p>
        </div>
        <Button
          variant="secondary"
          size="sm"
          icon={<RefreshCw className="size-3.5" />}
          onClick={fetchFriends}
          loading={loading}
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
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {friends.map((friend, idx) => (
            <Card key={friend.gid ?? idx}>
              <div className="flex items-center gap-3">
                <div className="size-10 rounded-full bg-surface-bright flex items-center justify-center shrink-0">
                  {friend.avatar_url ? (
                    <img
                      src={friend.avatar_url}
                      alt=""
                      className="size-10 rounded-full object-cover"
                    />
                  ) : (
                    <Users className="size-5 text-on-surface-muted" />
                  )}
                </div>
                <div className="min-w-0">
                  <p className="text-sm font-medium truncate">
                    {friend.name ?? `好友 #${friend.gid}`}
                  </p>
                  {friend.level != null && (
                    <p className="text-xs text-on-surface-muted">
                      Lv.{friend.level}
                    </p>
                  )}
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
