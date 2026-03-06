import { useState, useEffect, useCallback } from "react";
import { ListTodo, Gift } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { useTauriEvent } from "../hooks/useTauriEvent";
import * as api from "../api";

interface TaskInfo {
  task_id?: number;
  name?: string;
  desc?: string;
  progress?: number;
  target?: number;
  status?: number;
  reward_desc?: string;
}

export default function TasksPage() {
  const [tasks, setTasks] = useState<TaskInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [claiming, setClaiming] = useState(false);

  const fetchTasks = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getTasks()) as { tasks?: TaskInfo[] };
      setTasks(res?.tasks ?? (Array.isArray(res) ? res : []));
    } catch (e) {
      console.error("Failed to load tasks:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const handleDataChanged = useCallback(
    (scope: string) => {
      if (scope === "tasks") fetchTasks();
    },
    [fetchTasks]
  );
  useTauriEvent("data-changed", handleDataChanged);

  const handleClaimAll = async () => {
    setClaiming(true);
    try {
      await api.claimAllTasks();
      await fetchTasks();
    } catch (e) {
      console.error("Claim failed:", e);
    } finally {
      setClaiming(false);
    }
  };

  const claimableCount = tasks.filter(
    (t) => t.status === 1 || (t.progress != null && t.target != null && t.progress >= t.target)
  ).length;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">任务</h1>
          <p className="text-sm text-on-surface-muted">
            共 {tasks.length} 个任务 | {claimableCount} 个可领取
          </p>
        </div>
        {claimableCount > 0 && (
          <Button
            size="sm"
            icon={<Gift className="size-3.5" />}
            onClick={handleClaimAll}
            loading={claiming}
          >
            全部领取
          </Button>
        )}
      </div>

      {tasks.length === 0 && !loading ? (
        <EmptyState
          icon={<ListTodo className="size-10" />}
          title="暂无任务"
          description="请先连接游戏服务器"
        />
      ) : (
        <div className="space-y-3">
          {tasks.map((task, idx) => {
            const done =
              task.status === 2 ||
              (task.progress != null &&
                task.target != null &&
                task.progress >= task.target &&
                task.status === 2);
            const claimable =
              task.status === 1 ||
              (task.progress != null &&
                task.target != null &&
                task.progress >= task.target &&
                task.status !== 2);

            return (
              <Card key={task.task_id ?? idx}>
                <div className="flex items-center gap-4">
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">
                      {task.name ?? `任务 #${task.task_id}`}
                    </p>
                    {task.desc && (
                      <p className="text-xs text-on-surface-muted truncate">
                        {task.desc}
                      </p>
                    )}
                    {task.progress != null && task.target != null && (
                      <div className="mt-2">
                        <div className="flex items-center justify-between text-xs mb-1">
                          <span className="text-on-surface-muted">进度</span>
                          <span className="font-medium">
                            {task.progress}/{task.target}
                          </span>
                        </div>
                        <div className="h-1.5 rounded-full bg-surface-bright overflow-hidden">
                          <div
                            className="h-full rounded-full bg-primary-500 transition-all"
                            style={{
                              width: `${Math.min(
                                100,
                                (task.progress / task.target) * 100
                              )}%`,
                            }}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                  <div className="shrink-0">
                    {done ? (
                      <span className="rounded-full bg-gray-100 px-2.5 py-1 text-xs font-medium text-gray-600">
                        已完成
                      </span>
                    ) : claimable ? (
                      <span className="rounded-full bg-primary-100 px-2.5 py-1 text-xs font-medium text-primary-700">
                        可领取
                      </span>
                    ) : (
                      <span className="rounded-full bg-surface-bright px-2.5 py-1 text-xs font-medium text-on-surface-muted">
                        进行中
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
