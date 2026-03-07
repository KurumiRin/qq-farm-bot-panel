import { useState, useEffect, useCallback } from "react";
import { Save, RotateCw, Copy, Check } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { PageHeader } from "../components/PageHeader";
import { SEEDS } from "../data/seeds";
import type {
  AutomationConfig,
  PlantingStrategy,
  FertilizerStrategy,
} from "../types";
import * as api from "../api";

// --- Sub-components ---

interface ToggleRowProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}

function ToggleRow({ label, description, checked, onChange }: ToggleRowProps) {
  return (
    <div className="flex items-center justify-between gap-4 py-2.5">
      <div className="flex items-baseline gap-1.5 min-w-0">
        <p className="text-[13px] font-medium leading-tight shrink-0">{label}</p>
        {description && (
          <p className="text-xs text-on-surface-muted leading-tight truncate">
            {description}
          </p>
        )}
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={(e) => {
          e.stopPropagation();
          onChange(!checked);
        }}
        className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors ${
          checked ? "bg-primary-500" : "bg-gray-300"
        }`}
      >
        <span
          className={`inline-block size-3.5 transform rounded-full bg-white transition-transform ${
            checked ? "translate-x-4.5" : "translate-x-1"
          }`}
        />
      </button>
    </div>
  );
}

interface InlineToggleProps {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}

function InlineToggle({ label, checked, onChange }: InlineToggleProps) {
  return (
    <button
      type="button"
      onClick={() => onChange(!checked)}
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs transition-colors ${
        checked
          ? "bg-primary-500/10 text-primary-600"
          : "bg-gray-100 text-on-surface-muted"
      }`}
    >
      <span
        className={`size-1.5 rounded-full ${
          checked ? "bg-primary-500" : "bg-gray-300"
        }`}
      />
      {label}
    </button>
  );
}

const inputClass =
  "w-full h-9 rounded-lg border border-border bg-surface-dim px-3 text-sm outline-none focus:border-primary-500 focus:outline-1 focus:outline-primary-500";

const selectClass = `${inputClass} appearance-none bg-[url('data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%2216%22%20height%3D%2216%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%20stroke%3D%22%23999%22%20stroke-width%3D%222%22%3E%3Cpath%20d%3D%22m6%209%206%206%206-6%22%2F%3E%3C%2Fsvg%3E')] bg-size-[16px] bg-position-[right_8px_center] bg-no-repeat pr-8`;

// --- Constants ---

const STRATEGY_OPTIONS: {
  value: PlantingStrategy;
  label: string;
}[] = [
  { value: "preferred", label: "指定种子" },
  { value: "level", label: "最高等级作物" },
  { value: "max_exp", label: "最大经验/时" },
  { value: "max_profit", label: "最大利润/时" },
];

const FERTILIZER_OPTIONS: { value: FertilizerStrategy; label: string }[] = [
  { value: "none", label: "不施肥" },
  { value: "both", label: "普通 + 有机" },
  { value: "normal", label: "仅普通化肥" },
  { value: "organic", label: "仅有机化肥" },
];

const defaultConfig: AutomationConfig = {
  planting_strategy: "preferred",
  preferred_seed_id: null,
  intervals: { farm_min: 30, farm_max: 60, friend_min: 120, friend_max: 300 },
  friend_quiet_hours: { enabled: false, start: "23:00", end: "07:00" },

  auto_harvest: true,
  auto_plant: true,
  auto_farm_manage: true,
  auto_water: true,
  auto_weed: true,
  auto_insecticide: true,
  fertilizer_strategy: "none",
  auto_land_upgrade: false,
  auto_farm_push: true,

  auto_sell: true,
  auto_claim_tasks: true,
  auto_claim_emails: true,
  auto_free_gifts: false,
  auto_share_reward: false,
  auto_vip_gift: false,
  auto_month_card: false,
  auto_open_server_gift: false,
  auto_fertilizer_gift: false,
  auto_fertilizer_buy: false,

  auto_steal: true,
  auto_help_water: true,
  auto_help_weed: true,
  auto_help_insecticide: true,
  auto_friend_bad: false,
  auto_friend_help_exp_limit: false,

  friend_blacklist: [],
};

function LoginCodeRow() {
  const [code, setCode] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    api.getLoginCode().then(setCode).catch(() => {});
  }, []);

  const handleCopy = async () => {
    if (!code) return;
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className="flex items-center justify-between gap-4 py-2.5">
      <div className="min-w-0">
        <p className="text-[13px] font-medium leading-tight">当前 Code</p>
        <p className="text-xs text-on-surface-muted leading-tight mt-0.5 truncate font-mono">
          {code || "未登录"}
        </p>
      </div>
      {code && (
        <Button
          size="sm"
          variant="ghost"
          icon={copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
          onClick={handleCopy}
        >
          {copied ? "已复制" : "复制"}
        </Button>
      )}
    </div>
  );
}

// --- Main Component ---

export default function SettingsPage() {
  const [config, setConfig] = useState<AutomationConfig>(defaultConfig);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const fetchConfig = useCallback(async () => {
    setLoading(true);
    try {
      const c = await api.getAutomationConfig();
      setConfig(c);
    } catch (e) {
      console.error("Failed to load config:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  const toggle = (key: keyof AutomationConfig) => {
    setConfig((prev) => ({ ...prev, [key]: !prev[key] }));
    setSaved(false);
  };

  const set = <K extends keyof AutomationConfig>(
    key: K,
    value: AutomationConfig[K]
  ) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
    setSaved(false);
  };

  const setInterval = (
    key: keyof AutomationConfig["intervals"],
    value: number
  ) => {
    setConfig((prev) => ({
      ...prev,
      intervals: { ...prev.intervals, [key]: value },
    }));
    setSaved(false);
  };

  const setQuietHours = (
    key: keyof AutomationConfig["friend_quiet_hours"],
    value: string | boolean
  ) => {
    setConfig((prev) => ({
      ...prev,
      friend_quiet_hours: { ...prev.friend_quiet_hours, [key]: value },
    }));
    setSaved(false);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await api.setAutomationConfig(config);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save config:", e);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <span className="size-6 animate-spin rounded-full border-2 border-primary-500 border-t-transparent" />
      </div>
    );
  }

  return (
    <div className="relative max-w-2xl">
      {/* Sticky header */}
      <div className="sticky top-0 z-10 bg-surface-dim pb-4">
        <PageHeader
          title="设置"
          tags={[{ label: "配置自动化行为" }]}
          actions={
            <Button
              size="sm"
              icon={saved ? undefined : <Save className="size-3.5" />}
              onClick={handleSave}
              loading={saving}
            >
              {saved ? "已保存" : "保存"}
            </Button>
          }
        />
        <div className="absolute bottom-0 left-0 right-0 h-4 bg-linear-to-b from-surface-dim to-transparent pointer-events-none" />
      </div>

      <div className="space-y-3">
        {/* ===== 巡查节奏 ===== */}
        <Card title="巡查节奏" collapsible>
          <div className="space-y-4">
            <div>
              <p className="text-[13px] font-medium mb-2">农场巡查</p>
              <div className="grid grid-cols-2 gap-3">
                {(
                  [
                    ["farm_min", "最小间隔"],
                    ["farm_max", "最大间隔"],
                  ] as const
                ).map(([key, label]) => (
                  <div key={key}>
                    <label className="text-xs text-on-surface-muted mb-1 block">
                      {label}
                      <span className="opacity-50"> (秒)</span>
                    </label>
                    <input
                      type="number"
                      value={config.intervals[key]}
                      onChange={(e) =>
                        setInterval(
                          key,
                          Math.max(1, Math.min(86400, parseInt(e.target.value) || 1))
                        )
                      }
                      min={1}
                      max={86400}
                      className={`${inputClass} [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none`}
                    />
                  </div>
                ))}
              </div>
            </div>

            <div className="border-t border-border pt-4">
              <p className="text-[13px] font-medium mb-2">好友巡查</p>
              <div className="grid grid-cols-2 gap-3">
                {(
                  [
                    ["friend_min", "最小间隔"],
                    ["friend_max", "最大间隔"],
                  ] as const
                ).map(([key, label]) => (
                  <div key={key}>
                    <label className="text-xs text-on-surface-muted mb-1 block">
                      {label}
                      <span className="opacity-50"> (秒)</span>
                    </label>
                    <input
                      type="number"
                      value={config.intervals[key]}
                      onChange={(e) =>
                        setInterval(
                          key,
                          Math.max(1, Math.min(86400, parseInt(e.target.value) || 1))
                        )
                      }
                      min={1}
                      max={86400}
                      className={`${inputClass} [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none`}
                    />
                  </div>
                ))}
              </div>
              <div className="mt-3 pt-3 border-t border-border">
                <div className="flex items-center justify-between gap-4">
                  <ToggleRow
                    label="好友静默时段"
                    description="静默期间不巡查好友农场"
                    checked={config.friend_quiet_hours.enabled}
                    onChange={(v) => setQuietHours("enabled", v)}
                  />
                  {config.friend_quiet_hours.enabled && (
                    <div className="flex items-center gap-2 shrink-0">
                      <input
                        type="time"
                        value={config.friend_quiet_hours.start}
                        onChange={(e) => setQuietHours("start", e.target.value)}
                        className={`${inputClass} w-28`}
                      />
                      <span className="text-on-surface-muted text-xs">至</span>
                      <input
                        type="time"
                        value={config.friend_quiet_hours.end}
                        onChange={(e) => setQuietHours("end", e.target.value)}
                        className={`${inputClass} w-28`}
                      />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        </Card>

        {/* ===== 农场自动化 ===== */}
        <Card title="农场自动化" collapsible>
          <div className="divide-y divide-border">
            <ToggleRow
              label="自动种植收获"
              description="自动收获成熟作物并重新播种"
              checked={config.auto_harvest}
              onChange={() => toggle("auto_harvest")}
            />
            <div>
              <ToggleRow
                label="自动打理农场"
                description="自动浇水、除草、除虫"
                checked={config.auto_farm_manage}
                onChange={() => toggle("auto_farm_manage")}
              />
              {config.auto_farm_manage && (
                <div className="flex flex-wrap gap-1.5 pb-2.5">
                  <InlineToggle label="浇水" checked={config.auto_water} onChange={() => toggle("auto_water")} />
                  <InlineToggle label="除草" checked={config.auto_weed} onChange={() => toggle("auto_weed")} />
                  <InlineToggle label="除虫" checked={config.auto_insecticide} onChange={() => toggle("auto_insecticide")} />
                </div>
              )}
            </div>
            <ToggleRow
              label="自动升级土地"
              description="有足够金币时自动升级"
              checked={config.auto_land_upgrade}
              onChange={() => toggle("auto_land_upgrade")}
            />
            <ToggleRow
              label="推送触发巡田"
              description="收到服务器推送时立即巡田"
              checked={config.auto_farm_push}
              onChange={() => toggle("auto_farm_push")}
            />
          </div>
          <div className="mt-1 pt-1">
            <label className="text-xs text-on-surface-muted mb-1 block">
              施肥策略
            </label>
            <select
              value={config.fertilizer_strategy}
              onChange={(e) =>
                set("fertilizer_strategy", e.target.value as FertilizerStrategy)
              }
              className={`${selectClass} max-w-xs`}
            >
              {FERTILIZER_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </Card>

        {/* ===== 社交 ===== */}
        <Card title="社交" collapsible>
          <div className="divide-y divide-border">
            <ToggleRow
              label="自动偷菜"
              description="自动偷取好友农场的成熟作物"
              checked={config.auto_steal}
              onChange={() => toggle("auto_steal")}
            />
            <div>
              <ToggleRow
                label="自动帮忙"
                description="帮好友浇水、除草、除虫"
                checked={config.auto_help_water}
                onChange={() => toggle("auto_help_water")}
              />
              {config.auto_help_water && (
                <div className="flex flex-wrap gap-1.5 pb-2.5">
                  <InlineToggle label="浇水" checked={config.auto_help_water} onChange={() => toggle("auto_help_water")} />
                  <InlineToggle label="除草" checked={config.auto_help_weed} onChange={() => toggle("auto_help_weed")} />
                  <InlineToggle label="除虫" checked={config.auto_help_insecticide} onChange={() => toggle("auto_help_insecticide")} />
                  <InlineToggle label="经验上限停止" checked={config.auto_friend_help_exp_limit} onChange={() => toggle("auto_friend_help_exp_limit")} />
                </div>
              )}
            </div>
            <ToggleRow
              label="自动捣乱"
              description="给好友农场放虫、放草"
              checked={config.auto_friend_bad}
              onChange={() => toggle("auto_friend_bad")}
            />
          </div>
        </Card>

        {/* ===== 种植策略 ===== */}
        <Card title="种植策略" collapsible>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="text-xs text-on-surface-muted mb-1 block">
                播种策略
              </label>
              <select
                value={config.planting_strategy}
                onChange={(e) =>
                  set("planting_strategy", e.target.value as PlantingStrategy)
                }
                className={selectClass}
              >
                {STRATEGY_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </div>

            {config.planting_strategy === "preferred" ? (
              <div>
                <label className="text-xs text-on-surface-muted mb-1 block">
                  优先种植种子
                </label>
                <select
                  value={config.preferred_seed_id ?? ""}
                  onChange={(e) => {
                    const v = e.target.value;
                    set("preferred_seed_id", v ? parseInt(v, 10) : null);
                  }}
                  className={selectClass}
                >
                  <option value="">自动选择</option>
                  {SEEDS.map((seed) => (
                    <option key={seed.id} value={seed.id}>
                      {seed.level}级 {seed.name} ({seed.price}金)
                    </option>
                  ))}
                </select>
              </div>
            ) : (
              <div>
                <label className="text-xs text-on-surface-muted mb-1 block">
                  策略选种预览
                </label>
                <div className={`${inputClass} flex items-center text-on-surface-muted`}>
                  由系统自动选择
                </div>
              </div>
            )}
          </div>
        </Card>

        {/* ===== 出售与领取 ===== */}
        <Card title="出售与领取" collapsible>
          <div className="divide-y divide-border">
            <ToggleRow
              label="自动卖果实"
              description="收获后自动出售"
              checked={config.auto_sell}
              onChange={() => toggle("auto_sell")}
            />
            <ToggleRow
              label="自动做任务"
              description="自动领取任务奖励"
              checked={config.auto_claim_tasks}
              onChange={() => toggle("auto_claim_tasks")}
            />
            <ToggleRow
              label="自动领取邮件"
              checked={config.auto_claim_emails}
              onChange={() => toggle("auto_claim_emails")}
            />
            <ToggleRow
              label="自动商城礼包"
              description="领取免费商城礼包"
              checked={config.auto_free_gifts}
              onChange={() => toggle("auto_free_gifts")}
            />
            <ToggleRow
              label="自动分享奖励"
              checked={config.auto_share_reward}
              onChange={() => toggle("auto_share_reward")}
            />
            <ToggleRow
              label="自动VIP礼包"
              checked={config.auto_vip_gift}
              onChange={() => toggle("auto_vip_gift")}
            />
            <ToggleRow
              label="自动月卡奖励"
              checked={config.auto_month_card}
              onChange={() => toggle("auto_month_card")}
            />
            <ToggleRow
              label="自动开服红包"
              checked={config.auto_open_server_gift}
              onChange={() => toggle("auto_open_server_gift")}
            />
            <ToggleRow
              label="自动填充化肥"
              description="自动从背包填充化肥槽"
              checked={config.auto_fertilizer_gift}
              onChange={() => toggle("auto_fertilizer_gift")}
            />
            <ToggleRow
              label="自动购买化肥"
              description="化肥不足时自动购买"
              checked={config.auto_fertilizer_buy}
              onChange={() => toggle("auto_fertilizer_buy")}
            />
          </div>
        </Card>
        {/* ===== 系统 ===== */}
        <Card title="系统" collapsible>
          <div className="divide-y divide-border">
            <LoginCodeRow />
            <div className="flex items-center justify-between py-2.5">
              <div>
                <p className="text-[13px] font-medium leading-tight">Code 接收服务</p>
                <p className="text-xs text-on-surface-muted leading-tight mt-0.5">
                  端口 7788，用于接收登录 code。端口被占用时可重启
                </p>
              </div>
              <Button
                size="sm"
                variant="ghost"
                icon={<RotateCw className="size-3.5" />}
                onClick={async () => {
                  try {
                    await api.restartCodeReceiver();
                  } catch (e) {
                    console.error("Failed to restart code receiver:", e);
                  }
                }}
              >
                重启
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
