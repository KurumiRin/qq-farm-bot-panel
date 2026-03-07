import { useState, useEffect, useCallback } from "react";
import { ListTodo, Gift, Star, Zap, Check, Lock, ChevronDown, ChevronUp } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";
import { useTauriEvent } from "../hooks/useTauriEvent";
import * as api from "../api";

interface RewardView {
  id: number;
  count: number;
  name: string;
}

interface TaskView {
  id: number;
  desc: string;
  progress: number;
  total_progress: number;
  is_claimed: boolean;
  is_unlocked: boolean;
  task_type: number;
  share_multiple: number;
  rewards: RewardView[];
}

interface ActiveRewardView {
  point_id: number;
  need_progress: number;
  status: number;
  rewards: RewardView[];
}

interface ActiveView {
  active_type: number;
  progress: number;
  rewards: ActiveRewardView[];
}

interface TasksData {
  growth_tasks: TaskView[];
  daily_tasks: TaskView[];
  tasks: TaskView[];
  actives: ActiveView[];
}

function RewardBadges({ rewards }: { rewards: RewardView[] }) {
  if (!rewards.length) return null;
  return (
    <div className="flex flex-wrap gap-1">
      {rewards.map((r, i) => (
        <span
          key={i}
          className="inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 text-[10px] font-medium bg-amber-500/10 text-amber-700 dark:text-amber-400"
        >
          {r.name} ×{r.count}
        </span>
      ))}
    </div>
  );
}

function TaskCard({ task }: { task: TaskView }) {
  const completed = task.progress >= task.total_progress && task.total_progress > 0;
  const claimable = completed && !task.is_claimed && task.is_unlocked;
  const claimed = task.is_claimed;
  const locked = !task.is_unlocked;
  const pct = Math.round((task.progress / Math.max(1, task.total_progress)) * 100);

  return (
    <div className="py-2">
      <div className="flex items-center gap-2">
        <p className="text-sm font-medium truncate flex-1">{task.desc || `任务 #${task.id}`}</p>
        {task.share_multiple > 1 && (
          <span className="rounded px-1 py-0.5 text-[10px] font-semibold bg-purple-500/15 text-purple-600 dark:text-purple-400">
            {task.share_multiple}x
          </span>
        )}
        {locked ? (
          <Lock className="size-3.5 text-on-surface-muted/40" />
        ) : claimed ? (
          <span className="rounded-full bg-green-100 dark:bg-green-900/30 px-2 py-0.5 text-[10px] font-medium text-green-700 dark:text-green-400">
            <Check className="size-3 inline -mt-0.5" /> 已领
          </span>
        ) : claimable ? (
          <span className="rounded-full bg-green-100 dark:bg-green-900/30 px-2 py-0.5 text-[10px] font-semibold text-green-700 dark:text-green-400 animate-pulse">
            可领取
          </span>
        ) : (
          <span className="text-[10px] text-on-surface-muted">{pct}%</span>
        )}
      </div>
      {task.total_progress > 0 && (
        <div className="mt-1.5">
          <div className="flex items-center justify-between text-[11px] mb-0.5">
            <span className="text-on-surface-muted">
              {task.progress}/{task.total_progress}
            </span>
            <RewardBadges rewards={task.rewards} />
          </div>
          <div className="h-1 rounded-full bg-surface-bright overflow-hidden">
            <div
              className={`h-full rounded-full transition-all ${
                completed
                  ? "bg-green-500"
                  : "bg-blue-500"
              }`}
              style={{
                width: `${Math.min(100, pct)}%`,
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function TaskSection({
  title,
  icon,
  tasks,
  defaultOpen = true,
}: {
  title: string;
  icon: React.ReactNode;
  tasks: TaskView[];
  defaultOpen?: boolean;
}) {
  const [open, setOpen] = useState(defaultOpen);
  const claimable = tasks.filter(
    (t) => t.is_unlocked && !t.is_claimed && t.progress >= t.total_progress && t.total_progress > 0
  ).length;
  const completed = tasks.filter((t) => t.is_claimed).length;

  if (!tasks.length) return null;

  return (
    <Card>
      <button
        className="flex items-center gap-2 w-full text-left"
        onClick={() => setOpen(!open)}
      >
        {icon}
        <span className="text-sm font-semibold flex-1">{title}</span>
        <span className="text-[11px] text-on-surface-muted">
          {completed}/{tasks.length}
          {claimable > 0 && (
            <span className="ml-1 text-green-600 dark:text-green-400 font-medium">
              ({claimable} 可领)
            </span>
          )}
        </span>
        {open ? <ChevronUp className="size-3.5 text-on-surface-muted" /> : <ChevronDown className="size-3.5 text-on-surface-muted" />}
      </button>
      {open && (
        <div className="mt-2 divide-y divide-border">
          {tasks.map((task) => (
            <TaskCard key={task.id} task={task} />
          ))}
        </div>
      )}
    </Card>
  );
}

function ActiveSection({ actives }: { actives: ActiveView[] }) {
  if (!actives.length) return null;

  return (
    <>
      {actives.map((active) => {
        const typeName = active.active_type === 1 ? "日活跃" : active.active_type === 2 ? "周活跃" : "活跃度";
        const maxProgress = active.rewards.length > 0
          ? Math.max(...active.rewards.map((r) => r.need_progress))
          : 100;
        const claimable = active.rewards.filter((r) => r.status === 2).length;

        return (
          <Card key={active.active_type}>
            <div className="flex items-center gap-2 mb-3">
              <Zap className="size-4 text-amber-500" />
              <span className="text-sm font-semibold flex-1">{typeName}</span>
              <span className="text-[11px] text-on-surface-muted">
                {active.progress}/{maxProgress}
                {claimable > 0 && (
                  <span className="ml-1 text-green-600 dark:text-green-400 font-medium">
                    ({claimable} 可领)
                  </span>
                )}
              </span>
            </div>
            <div className="relative h-2 rounded-full bg-surface-bright overflow-hidden mb-3">
              <div
                className="h-full rounded-full bg-amber-500 transition-all"
                style={{ width: `${Math.min(100, (active.progress / maxProgress) * 100)}%` }}
              />
              {active.rewards.map((r) => (
                <div
                  key={r.point_id}
                  className="absolute top-0 h-full w-px bg-on-surface-muted/20"
                  style={{ left: `${(r.need_progress / maxProgress) * 100}%` }}
                />
              ))}
            </div>
            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2">
              {active.rewards.map((r) => {
                const reached = active.progress >= r.need_progress;
                const isClaimable = r.status === 2;
                return (
                  <div
                    key={r.point_id}
                    className={`rounded-lg border p-2 text-center text-[11px] ${
                      isClaimable
                        ? "border-green-500/50 bg-green-50 dark:bg-green-950/20"
                        : reached
                          ? "border-amber-500/30 bg-amber-50/50 dark:bg-amber-950/10"
                          : "border-border bg-surface"
                    }`}
                  >
                    <div className="font-semibold mb-1">
                      {r.need_progress} 点
                      {isClaimable && (
                        <span className="ml-1 text-green-600 dark:text-green-400">✓</span>
                      )}
                    </div>
                    <RewardBadges rewards={r.rewards} />
                  </div>
                );
              })}
            </div>
          </Card>
        );
      })}
    </>
  );
}

export default function TasksPage() {
  const [data, setData] = useState<TasksData | null>(null);
  const [loading, setLoading] = useState(false);
  const [claiming, setClaiming] = useState(false);

  const fetchTasks = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getTasks()) as TasksData;
      setData(res);
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

  const handleStatusChanged = useCallback(
    (payload: { connection: string }) => {
      if (payload.connection === "LoggedIn") fetchTasks();
    },
    [fetchTasks]
  );
  useTauriEvent("status-changed", handleStatusChanged);

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

  const allTasks = [
    ...(data?.growth_tasks ?? []),
    ...(data?.daily_tasks ?? []),
    ...(data?.tasks ?? []),
  ];
  const claimableCount = allTasks.filter(
    (t) => t.is_unlocked && !t.is_claimed && t.progress >= t.total_progress && t.total_progress > 0
  ).length;
  const activeClaimable = (data?.actives ?? []).reduce(
    (sum, a) => sum + a.rewards.filter((r) => r.status === 2).length,
    0
  );
  const totalClaimable = claimableCount + activeClaimable;

  const isEmpty = !data || (allTasks.length === 0 && (data?.actives ?? []).length === 0);

  return (
    <div className="space-y-4">
      <PageHeader
        title="任务"
        tags={data ? [
          { label: "任务", value: allTasks.length },
          { label: "可领取", value: totalClaimable, cls: "bg-green-500/10 text-green-700 dark:text-green-400", hidden: totalClaimable === 0 },
        ] : [{ label: "加载中..." }]}
        actions={totalClaimable > 0 ? (
          <Button
            size="sm"
            icon={<Gift className="size-3.5" />}
            onClick={handleClaimAll}
            loading={claiming}
          >
            全部领取 ({totalClaimable})
          </Button>
        ) : undefined}
      />

      {isEmpty && !loading ? (
        <EmptyState
          icon={<ListTodo className="size-10" />}
          title="暂无任务"
          description="请先连接游戏服务器"
        />
      ) : (
        <div className="space-y-4">
          <TaskSection
            title="每日任务"
            icon={<Star className="size-4 text-blue-500" />}
            tasks={data?.daily_tasks ?? []}
          />
          <TaskSection
            title="成长任务"
            icon={<Zap className="size-4 text-green-500" />}
            tasks={data?.growth_tasks ?? []}
          />
          {(data?.tasks ?? []).length > 0 && (
            <TaskSection
              title="其他任务"
              icon={<ListTodo className="size-4 text-on-surface-muted" />}
              tasks={data?.tasks ?? []}
              defaultOpen={false}
            />
          )}
          <ActiveSection actives={data?.actives ?? []} />
        </div>
      )}
    </div>
  );
}
