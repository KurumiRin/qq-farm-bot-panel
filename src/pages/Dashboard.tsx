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
        <h1 className="text-xl font-bold">仪表盘</h1>
        <p className="text-sm text-on-surface-muted">
          {isOnline ? `欢迎回来，${user.name}` : "未连接"}
        </p>
      </div>

      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <Card>
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-yellow-50 p-2">
              <Coins className="size-5 text-yellow-600" />
            </div>
            <div>
              <p className="text-xs text-on-surface-muted">金币</p>
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
              <p className="text-xs text-on-surface-muted">等级</p>
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
              <p className="text-xs text-on-surface-muted">经验</p>
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
              <p className="text-xs text-on-surface-muted">点券</p>
              <p className="text-lg font-bold">{user.coupon}</p>
            </div>
          </div>
        </Card>
      </div>

      <Card title="本次统计">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
          <StatItem
            icon={<Sprout className="size-4" />}
            label="收获"
            value={stats.harvests}
          />
          <StatItem
            icon={<Sprout className="size-4" />}
            label="播种"
            value={stats.plants}
          />
          <StatItem
            icon={<Droplets className="size-4" />}
            label="浇水"
            value={stats.waters}
          />
          <StatItem
            icon={<Scissors className="size-4" />}
            label="除草"
            value={stats.weeds_removed}
          />
          <StatItem
            icon={<Bug className="size-4" />}
            label="除虫"
            value={stats.insects_removed}
          />
          <StatItem
            icon={<Swords className="size-4" />}
            label="偷菜"
            value={stats.steals}
          />
          <StatItem
            icon={<Users className="size-4" />}
            label="帮浇水"
            value={stats.help_waters}
          />
          <StatItem
            icon={<Coins className="size-4" />}
            label="获得金币"
            value={stats.gold_earned.toLocaleString()}
          />
          <StatItem
            icon={<Package className="size-4" />}
            label="售出物品"
            value={stats.items_sold}
          />
          <StatItem
            icon={<ListChecks className="size-4" />}
            label="领取任务"
            value={stats.tasks_claimed}
          />
          <StatItem
            icon={<Mail className="size-4" />}
            label="领取邮件"
            value={stats.emails_claimed}
          />
        </div>
      </Card>
    </div>
  );
}
