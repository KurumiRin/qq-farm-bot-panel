import { NavLink, Outlet } from "react-router-dom";
import { clsx } from "clsx";
import {
  LayoutDashboard,
  Sprout,
  Users,
  Package,
  ListTodo,
  Settings,
  LogOut,
  Wifi,
  WifiOff,
  Menu,
  X,
} from "lucide-react";
import { useState, useCallback } from "react";
import { useStatus } from "../hooks/useStatus";
import * as api from "../api";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "仪表盘" },
  { to: "/farm", icon: Sprout, label: "农场" },
  { to: "/friends", icon: Users, label: "好友" },
  { to: "/inventory", icon: Package, label: "仓库" },
  { to: "/tasks", icon: ListTodo, label: "任务" },
  { to: "/settings", icon: Settings, label: "设置" },
] as const;

function ConnectionBadge({ connected }: { connected: boolean }) {
  return (
    <div
      className={clsx(
        "flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium",
        connected
          ? "bg-primary-100 text-primary-700"
          : "bg-red-100 text-red-700"
      )}
    >
      {connected ? <Wifi className="size-3" /> : <WifiOff className="size-3" />}
      {connected ? "已连接" : "未连接"}
    </div>
  );
}

export default function Layout() {
  const [mobileOpen, setMobileOpen] = useState(false);
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

  const closeMobile = () => setMobileOpen(false);

  const sidebarContent = (mobile: boolean) => (
    <>
      <div className="px-5 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Sprout className="size-7 text-primary-500" />
            <span className="text-lg font-semibold">Farm Pilot</span>
          </div>
          {mobile && (
            <button onClick={closeMobile}>
              <X className="size-5 text-on-surface-muted" />
            </button>
          )}
        </div>
        {status?.user && status.user.gid !== 0 && (
          <div className="mt-3 rounded-lg bg-surface-bright px-3 py-2">
            <p className="text-sm font-medium truncate">{status.user.name}</p>
            <p className="text-xs text-on-surface-muted">
              Lv.{status.user.level} | 金币: {status.user.gold.toLocaleString()}
            </p>
          </div>
        )}
      </div>

      <nav className="flex-1 px-3 py-1 space-y-0.5 overflow-y-auto">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            onClick={mobile ? closeMobile : undefined}
            className={({ isActive }) =>
              clsx(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary-50 text-primary-700"
                  : "text-on-surface-muted hover:bg-surface-bright hover:text-on-surface"
              )
            }
          >
            <Icon className="size-4" />
            {label}
          </NavLink>
        ))}
      </nav>

      <div className="px-3 py-3 border-t border-border space-y-2">
        <ConnectionBadge connected={isConnected} />
        {isConnected && (
          <button
            onClick={handleDisconnect}
            className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
          >
            <LogOut className="size-4" />
            断开连接
          </button>
        )}
      </div>
    </>
  );

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Desktop Sidebar */}
      <aside className="hidden md:flex md:w-60 flex-col border-r border-border bg-surface">
        {sidebarContent(false)}
      </aside>

      {/* Mobile overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/30 md:hidden"
          onClick={closeMobile}
        />
      )}

      {/* Mobile sidebar */}
      <aside
        className={clsx(
          "fixed inset-y-0 left-0 z-50 w-64 flex flex-col bg-surface border-r border-border transition-transform duration-200 md:hidden",
          mobileOpen ? "translate-x-0" : "-translate-x-full"
        )}
      >
        {sidebarContent(true)}
      </aside>

      {/* Main content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Mobile header */}
        <header className="flex items-center gap-3 px-4 py-3 border-b border-border bg-surface md:hidden">
          <button onClick={() => setMobileOpen(true)}>
            <Menu className="size-5 text-on-surface" />
          </button>
          <span className="font-semibold">Farm Pilot</span>
          <div className="ml-auto">
            <ConnectionBadge connected={isConnected} />
          </div>
        </header>

        <main className="flex-1 overflow-y-auto p-4 md:p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
