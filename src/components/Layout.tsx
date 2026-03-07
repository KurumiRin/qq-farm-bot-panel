import { NavLink, useLocation } from "react-router-dom";
import { clsx } from "clsx";
import {
  LayoutDashboard,
  Sprout,
  Users,
  Package,
  ListTodo,
  Settings,
  ScrollText,
  LogOut,
  Circle,
  PlugZap,
  X,
  Coins,
} from "lucide-react";
import { getLevelProgress } from "../data/levelExp";

function formatCompact(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, "")}M`;
  if (n >= 10_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
  return n.toLocaleString();
}
import { lazy, Suspense, useCallback, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useStatus } from "../hooks/useStatus";
import { useIndicator } from "../hooks/useIndicator";
import * as api from "../api";

const DashboardPage = lazy(() => import("../pages/Dashboard"));
const FarmPage = lazy(() => import("../pages/Farm"));
const FriendsPage = lazy(() => import("../pages/Friends"));
const InventoryPage = lazy(() => import("../pages/Inventory"));
const TasksPage = lazy(() => import("../pages/Tasks"));
const SettingsPage = lazy(() => import("../pages/Settings"));
const LogsPage = lazy(() => import("../pages/Logs"));

const startDrag = (e: React.MouseEvent) => {
  e.preventDefault();
  getCurrentWindow().startDragging();
};

const DRAG_CLASS = "absolute inset-x-0 top-0 h-11 z-20";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "仪表盘" },
  { to: "/farm", icon: Sprout, label: "农场" },
  { to: "/friends", icon: Users, label: "好友" },
  { to: "/inventory", icon: Package, label: "仓库" },
  { to: "/tasks", icon: ListTodo, label: "任务" },
  { to: "/settings", icon: Settings, label: "设置" },
  { to: "/logs", icon: ScrollText, label: "日志" },
] as const;

const pages: { path: string; component: React.LazyExoticComponent<React.ComponentType> }[] = [
  { path: "/", component: DashboardPage },
  { path: "/farm", component: FarmPage },
  { path: "/friends", component: FriendsPage },
  { path: "/inventory", component: InventoryPage },
  { path: "/tasks", component: TasksPage },
  { path: "/settings", component: SettingsPage },
  { path: "/logs", component: LogsPage },
];

function NavList({ items }: { items: typeof navItems }) {
  const location = useLocation();

  const activeIndex = items.findIndex(({ to }) =>
    to === "/" ? location.pathname === "/" : location.pathname.startsWith(to)
  );

  const { containerRef: navRef, indicator } = useIndicator(activeIndex, "y");

  return (
    <nav ref={navRef} className="relative flex-1 px-3 space-y-0.5 overflow-y-auto">
      {/* Sliding indicator */}
      <div
        className={clsx(
          "absolute left-3 right-3 rounded-lg bg-primary-500 shadow-sm pointer-events-none",
          !indicator.ready ? "opacity-0" : indicator.animate ? "transition-all duration-300 ease-in-out" : ""
        )}
        style={{ top: indicator.offset, height: indicator.size }}
      />
      {items.map(({ to, icon: Icon, label }) => (
        <NavLink
          key={to}
          to={to}
          end={to === "/"}
          className={({ isActive }) =>
            clsx(
              "relative z-1 flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] font-medium transition-colors",
              isActive
                ? "text-white"
                : "text-on-surface-muted hover:bg-surface-bright hover:text-on-surface"
            )
          }
        >
          <Icon className="size-4" />
          {label}
        </NavLink>
      ))}
    </nav>
  );
}

function KeepAlivePages() {
  const location = useLocation();

  return (
    <Suspense>
      {pages.map(({ path, component: Page }) => (
        <div
          key={path}
          className={clsx(location.pathname !== path && "hidden")}
        >
          <Page />
        </div>
      ))}
    </Suspense>
  );
}

function CodeDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [code, setCode] = useState("");
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!open) return null;

  const handleConnect = async () => {
    const trimmed = code.trim();
    if (!trimmed) return;
    setConnecting(true);
    setError(null);
    try {
      await api.connectAndLogin(trimmed);
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center animate-[fade-in_150ms_ease-out] bg-black/40" onClick={onClose}>
      <div className="w-80 rounded-xl bg-surface border border-border p-4 shadow-lg space-y-3 animate-[scale-in_150ms_ease-out]" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold">输入登录 Code</h3>
          <button onClick={onClose} className="text-on-surface-muted hover:text-on-surface transition-colors">
            <X className="size-4" />
          </button>
        </div>
        <input
          autoFocus
          value={code}
          onChange={(e) => setCode(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleConnect()}
          placeholder="粘贴 Code..."
          className="w-full rounded-lg border border-border bg-surface-dim px-3 py-2 text-sm outline-none focus:border-primary-500"
        />
        {error && <p className="text-xs text-red-500">{error}</p>}
        <button
          onClick={handleConnect}
          disabled={!code.trim() || connecting}
          className="flex w-full items-center justify-center gap-1.5 rounded-lg bg-primary-500 px-3 py-2 text-xs font-medium text-white hover:bg-primary-600 disabled:opacity-50 transition-colors"
        >
          {connecting ? "连接中..." : "连接"}
        </button>
      </div>
    </div>
  );
}

export default function Layout() {
  const { status } = useStatus();
  const [codeDialogOpen, setCodeDialogOpen] = useState(false);

  const isConnected =
    status?.connection === "LoggedIn" || status?.connection === "Connected";

  const handleDisconnect = useCallback(async () => {
    try {
      await api.disconnect();
    } catch (e) {
      console.error("Disconnect failed:", e);
    }
  }, []);

  return (
    <div className="flex h-screen overflow-hidden bg-surface-dim">
      {/* Sidebar */}
      <aside className="relative flex w-56 shrink-0 flex-col bg-surface border-r border-border">
        <div className={DRAG_CLASS} onMouseDown={startDrag} />
        <div className="h-11 shrink-0" />

        {/* Brand */}
        <div className="px-4 pb-4">
          <div className="flex items-center gap-2.5">
            <div className="flex size-8 items-center justify-center rounded-lg bg-primary-500 shrink-0">
              <Sprout className="size-4.5 text-white" />
            </div>
            <div>
              <p className="text-sm font-semibold leading-none">Farm Bot Panel</p>
              <p className="mt-0.5 text-[11px] text-on-surface-muted leading-none">QQ 农场助手</p>
            </div>
          </div>
        </div>

        {/* User card */}
        {status?.user && status.user.gid !== 0 && (
          <div className="mx-3 mb-3 rounded-lg bg-surface-bright/70 px-3 py-2.5">
            <div className="flex items-center gap-2.5">
              <div className="size-8 rounded-full bg-primary-100 flex items-center justify-center shrink-0 overflow-hidden">
                {status.user.avatar_url ? (
                  <img src={status.user.avatar_url} alt="" className="size-8 rounded-full object-cover" />
                ) : (
                  <span className="text-xs font-bold text-primary-700">
                    {status.user.name.charAt(0)}
                  </span>
                )}
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1">
                  <span className="text-xs font-medium truncate">{status.user.name}</span>
                  <span className="text-[10px] text-on-surface-muted shrink-0">Lv.{status.user.level}</span>
                </div>
                <div className="flex items-center gap-0.5 text-[11px] text-on-surface-muted">
                  <Coins className="size-3 text-yellow-500" />
                  <span>{formatCompact(status.user.gold)}</span>
                </div>
                {(() => {
                  const { ratio, current, needed } = getLevelProgress(status.user.level, status.user.exp);
                  return (
                    <div className="mt-1.5 flex items-center gap-1.5">
                      <div className="flex-1 h-1 rounded-full bg-surface overflow-hidden">
                        <div className="h-full rounded-full bg-primary-400 transition-all" style={{ width: `${ratio * 100}%` }} />
                      </div>
                      <span className="text-[9px] text-on-surface-muted/60 shrink-0 tabular-nums">
                        {formatCompact(current)}/{formatCompact(needed)}
                      </span>
                    </div>
                  );
                })()}
              </div>
            </div>
          </div>
        )}

        {/* Nav */}
        <NavList items={navItems} />

        {/* Bottom status */}
        <div className="mx-3 my-3 rounded-lg border border-border p-3 space-y-2.5">
          <div className="flex items-center gap-2">
            <Circle
              className={clsx(
                "size-2",
                isConnected ? "fill-primary-500 text-primary-500" : "fill-red-400 text-red-400"
              )}
            />
            <span className="text-xs text-on-surface-muted">
              {isConnected ? "已连接到服务器" : "未连接"}
            </span>
          </div>
          {isConnected ? (
            <button
              onClick={handleDisconnect}
              className="flex w-full items-center justify-center gap-1.5 rounded-md border border-red-200 px-2.5 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 transition-colors"
            >
              <LogOut className="size-3" />
              断开连接
            </button>
          ) : (
            <button
              onClick={() => setCodeDialogOpen(true)}
              className="flex w-full items-center justify-center gap-1.5 rounded-md bg-primary-500 px-2.5 py-1.5 text-xs font-medium text-white hover:bg-primary-600 transition-colors"
            >
              <PlugZap className="size-3" />
              输入 Code 连接
            </button>
          )}
        </div>
        <CodeDialog open={codeDialogOpen} onClose={() => setCodeDialogOpen(false)} />
      </aside>

      {/* Main content */}
      <div className="relative flex flex-1 flex-col overflow-hidden">
        <div className={DRAG_CLASS} onMouseDown={startDrag} />

        <main className="flex-1 overflow-y-auto px-6 pb-6">
          <KeepAlivePages />
        </main>
      </div>
    </div>
  );
}
