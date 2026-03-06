import { NavLink, Outlet } from "react-router-dom";
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
} from "lucide-react";
import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useStatus } from "../hooks/useStatus";
import * as api from "../api";

const startDrag = (e: React.MouseEvent) => {
  e.preventDefault();
  getCurrentWindow().startDragging();
};

const DRAG_CLASS = "absolute inset-x-0 top-0 h-11 z-10";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "仪表盘" },
  { to: "/farm", icon: Sprout, label: "农场" },
  { to: "/friends", icon: Users, label: "好友" },
  { to: "/inventory", icon: Package, label: "仓库" },
  { to: "/tasks", icon: ListTodo, label: "任务" },
  { to: "/settings", icon: Settings, label: "设置" },
  { to: "/logs", icon: ScrollText, label: "日志" },
] as const;

export default function Layout() {
  const { status } = useStatus();

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
              <p className="text-sm font-semibold leading-none">Farm Pilot</p>
              <p className="mt-0.5 text-[11px] text-on-surface-muted leading-none">QQ 农场助手</p>
            </div>
          </div>
        </div>

        {/* User card */}
        {status?.user && status.user.gid !== 0 && (
          <div className="mx-3 mb-3 rounded-lg bg-surface-bright/70 px-3 py-2.5">
            <div className="flex items-center gap-2.5">
              <div className="size-8 rounded-full bg-primary-100 flex items-center justify-center shrink-0">
                <span className="text-xs font-bold text-primary-700">
                  {status.user.name.charAt(0)}
                </span>
              </div>
              <div className="min-w-0">
                <p className="text-xs font-medium truncate">{status.user.name}</p>
                <p className="text-[11px] text-on-surface-muted">
                  Lv.{status.user.level} · {status.user.gold.toLocaleString()} 金币
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Nav */}
        <nav className="flex-1 px-3 space-y-0.5 overflow-y-auto">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                clsx(
                  "flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] font-medium transition-colors",
                  isActive
                    ? "bg-primary-500 text-white shadow-sm"
                    : "text-on-surface-muted hover:bg-surface-bright hover:text-on-surface"
                )
              }
            >
              <Icon className="size-4" />
              {label}
            </NavLink>
          ))}
        </nav>

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
          {isConnected && (
            <button
              onClick={handleDisconnect}
              className="flex w-full items-center justify-center gap-1.5 rounded-md border border-red-200 px-2.5 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 transition-colors"
            >
              <LogOut className="size-3" />
              断开连接
            </button>
          )}
        </div>
      </aside>

      {/* Main content */}
      <div className="relative flex flex-1 flex-col overflow-hidden">
        <div className={DRAG_CLASS} onMouseDown={startDrag} />
        <div className="h-11 shrink-0" />

        <main className="flex-1 overflow-y-auto px-6 py-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
