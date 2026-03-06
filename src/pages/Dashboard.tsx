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
    <div className="flex items-center gap-2.5">
      <div className="rounded-lg bg-surface-bright p-2 text-on-surface-muted">
        {icon}
      </div>
      <div className="min-w-0">
        <p className="text-xs text-on-surface-muted">{label}</p>
        <p className="text-sm font-semibold truncate">{value}</p>
      </div>
    </div>
  );
}

const infoItems = [
  { key: "gold", label: "金币", color: "bg-yellow-50 text-yellow-600", icon: Coins, format: true },
  { key: "level", label: "等级", color: "bg-blue-50 text-blue-600", icon: TrendingUp, format: false },
  { key: "exp", label: "经验", color: "bg-purple-50 text-purple-600", icon: TrendingUp, format: true },
  { key: "coupon", label: "点券", color: "bg-pink-50 text-pink-600", icon: Coins, format: false },
] as const;

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

      {/* Info cards - 自适应宽度 */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        {infoItems.map(({ key, label, color, icon: Icon, format }) => {
          const val = user[key];
          return (
            <Card key={key} className="p-3!">
              <div className="flex items-center gap-2.5">
                <div className={`rounded-lg p-1.5 shrink-0 ${color}`}>
                  <Icon className="size-4" />
                </div>
                <div className="min-w-0">
                  <p className="text-[11px] text-on-surface-muted">{label}</p>
                  <p className="text-sm font-bold truncate" title={String(val)}>
                    {format ? Number(val).toLocaleString() : val}
                  </p>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* Stats */}
      <Card title="本次统计">
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          <StatItem icon={<Sprout className="size-4" />} label="收获" value={stats.harvests} />
          <StatItem icon={<Sprout className="size-4" />} label="播种" value={stats.plants} />
          <StatItem icon={<Droplets className="size-4" />} label="浇水" value={stats.waters} />
          <StatItem icon={<Scissors className="size-4" />} label="除草" value={stats.weeds_removed} />
          <StatItem icon={<Bug className="size-4" />} label="除虫" value={stats.insects_removed} />
          <StatItem icon={<Swords className="size-4" />} label="偷菜" value={stats.steals} />
          <StatItem icon={<Users className="size-4" />} label="帮浇水" value={stats.help_waters} />
          <StatItem icon={<Coins className="size-4" />} label="获得金币" value={stats.gold_earned.toLocaleString()} />
          <StatItem icon={<Package className="size-4" />} label="售出物品" value={stats.items_sold} />
          <StatItem icon={<ListChecks className="size-4" />} label="领取任务" value={stats.tasks_claimed} />
          <StatItem icon={<Mail className="size-4" />} label="领取邮件" value={stats.emails_claimed} />
        </div>
      </Card>
    </div>
  );
}
