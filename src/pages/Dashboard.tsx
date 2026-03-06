import {
  Sprout,
  Coins,
  TrendingUp,
  Users,
  Swords,
  Droplets,
  Bug,
  Scissors,
  Package,
  ListChecks,
  Mail,
} from "lucide-react";
import { Card } from "../components/Card";
import { useStatus } from "../hooks/useStatus";

interface StatItemProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
}

function StatItem({ icon, label, value }: StatItemProps) {
  return (
    <div className="flex items-center gap-3">
      <div className="rounded-lg bg-surface-bright p-2 text-on-surface-muted">
        {icon}
      </div>
      <div>
        <p className="text-xs text-on-surface-muted">{label}</p>
        <p className="text-sm font-semibold">{value}</p>
      </div>
    </div>
  );
}

export default function DashboardPage() {
  const { status } = useStatus();

  if (!status) {
    return (
      <div className="flex items-center justify-center py-20">
        <span className="size-6 animate-spin rounded-full border-2 border-primary-500 border-t-transparent" />
      </div>
    );
  }

  const { user, stats, connection } = status;
  const isOnline = connection === "LoggedIn";

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold">Dashboard</h1>
        <p className="text-sm text-on-surface-muted">
          {isOnline
            ? `Welcome back, ${user.name}`
            : "Not connected"}
        </p>
      </div>

      {/* User info cards */}
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <Card>
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-yellow-50 p-2">
              <Coins className="size-5 text-yellow-600" />
            </div>
            <div>
              <p className="text-xs text-on-surface-muted">Gold</p>
              <p className="text-lg font-bold">{user.gold.toLocaleString()}</p>
            </div>
          </div>
        </Card>
        <Card>
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-blue-50 p-2">
              <TrendingUp className="size-5 text-blue-600" />
            </div>
            <div>
              <p className="text-xs text-on-surface-muted">Level</p>
              <p className="text-lg font-bold">{user.level}</p>
            </div>
          </div>
        </Card>
        <Card>
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-purple-50 p-2">
              <TrendingUp className="size-5 text-purple-600" />
            </div>
            <div>
              <p className="text-xs text-on-surface-muted">EXP</p>
              <p className="text-lg font-bold">{user.exp.toLocaleString()}</p>
            </div>
          </div>
        </Card>
        <Card>
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-pink-50 p-2">
              <Coins className="size-5 text-pink-600" />
            </div>
            <div>
              <p className="text-xs text-on-surface-muted">Coupons</p>
              <p className="text-lg font-bold">{user.coupon}</p>
            </div>
          </div>
        </Card>
      </div>

      {/* Stats */}
      <Card title="Session Statistics">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
          <StatItem
            icon={<Sprout className="size-4" />}
            label="Harvests"
            value={stats.harvests}
          />
          <StatItem
            icon={<Sprout className="size-4" />}
            label="Plants"
            value={stats.plants}
          />
          <StatItem
            icon={<Droplets className="size-4" />}
            label="Watered"
            value={stats.waters}
          />
          <StatItem
            icon={<Scissors className="size-4" />}
            label="Weeds Removed"
            value={stats.weeds_removed}
          />
          <StatItem
            icon={<Bug className="size-4" />}
            label="Insects Removed"
            value={stats.insects_removed}
          />
          <StatItem
            icon={<Swords className="size-4" />}
            label="Steals"
            value={stats.steals}
          />
          <StatItem
            icon={<Users className="size-4" />}
            label="Help Waters"
            value={stats.help_waters}
          />
          <StatItem
            icon={<Coins className="size-4" />}
            label="Gold Earned"
            value={stats.gold_earned.toLocaleString()}
          />
          <StatItem
            icon={<Package className="size-4" />}
            label="Items Sold"
            value={stats.items_sold}
          />
          <StatItem
            icon={<ListChecks className="size-4" />}
            label="Tasks Claimed"
            value={stats.tasks_claimed}
          />
          <StatItem
            icon={<Mail className="size-4" />}
            label="Emails Claimed"
            value={stats.emails_claimed}
          />
        </div>
      </Card>
    </div>
  );
}
