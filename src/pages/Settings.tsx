import { useState, useEffect, useCallback } from "react";
import { Save } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import type { AutomationConfig } from "../types";
import * as api from "../api";

interface ToggleRowProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}

function ToggleRow({ label, description, checked, onChange }: ToggleRowProps) {
  return (
    <label className="flex items-center justify-between py-2 cursor-pointer">
      <div>
        <p className="text-sm font-medium">{label}</p>
        {description && (
          <p className="text-xs text-on-surface-muted">{description}</p>
        )}
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
          checked ? "bg-primary-500" : "bg-gray-300"
        }`}
      >
        <span
          className={`inline-block size-3.5 transform rounded-full bg-white transition-transform ${
            checked ? "translate-x-4.5" : "translate-x-1"
          }`}
        />
      </button>
    </label>
  );
}

const defaultConfig: AutomationConfig = {
  auto_harvest: true,
  auto_plant: true,
  auto_water: true,
  auto_weed: true,
  auto_insecticide: true,
  auto_fertilize: false,
  auto_sell: true,
  auto_steal: true,
  auto_help_water: true,
  auto_help_weed: true,
  auto_help_insecticide: true,
  auto_claim_tasks: true,
  auto_claim_emails: true,
  preferred_seed_id: null,
  friend_blacklist: [],
};

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
    <div className="space-y-6 max-w-2xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">Settings</h1>
          <p className="text-sm text-on-surface-muted">
            Configure automation behavior
          </p>
        </div>
        <Button
          size="sm"
          icon={
            saved ? undefined : <Save className="size-3.5" />
          }
          onClick={handleSave}
          loading={saving}
        >
          {saved ? "Saved!" : "Save"}
        </Button>
      </div>

      <Card title="Farm Automation">
        <div className="divide-y divide-border">
          <ToggleRow
            label="Auto Harvest"
            description="Automatically harvest mature crops"
            checked={config.auto_harvest}
            onChange={() => toggle("auto_harvest")}
          />
          <ToggleRow
            label="Auto Plant"
            description="Re-plant after harvest"
            checked={config.auto_plant}
            onChange={() => toggle("auto_plant")}
          />
          <ToggleRow
            label="Auto Water"
            description="Water dry crops automatically"
            checked={config.auto_water}
            onChange={() => toggle("auto_water")}
          />
          <ToggleRow
            label="Auto Weed"
            description="Remove weeds automatically"
            checked={config.auto_weed}
            onChange={() => toggle("auto_weed")}
          />
          <ToggleRow
            label="Auto Insecticide"
            description="Remove insects automatically"
            checked={config.auto_insecticide}
            onChange={() => toggle("auto_insecticide")}
          />
          <ToggleRow
            label="Auto Fertilize"
            description="Use fertilizer on crops"
            checked={config.auto_fertilize}
            onChange={() => toggle("auto_fertilize")}
          />
        </div>
      </Card>

      <Card title="Sell & Collect">
        <div className="divide-y divide-border">
          <ToggleRow
            label="Auto Sell"
            description="Sell fruits automatically"
            checked={config.auto_sell}
            onChange={() => toggle("auto_sell")}
          />
          <ToggleRow
            label="Auto Claim Tasks"
            description="Claim task rewards"
            checked={config.auto_claim_tasks}
            onChange={() => toggle("auto_claim_tasks")}
          />
          <ToggleRow
            label="Auto Claim Emails"
            description="Claim email rewards"
            checked={config.auto_claim_emails}
            onChange={() => toggle("auto_claim_emails")}
          />
        </div>
      </Card>

      <Card title="Social">
        <div className="divide-y divide-border">
          <ToggleRow
            label="Auto Steal"
            description="Steal from friends' farms"
            checked={config.auto_steal}
            onChange={() => toggle("auto_steal")}
          />
          <ToggleRow
            label="Auto Help Water"
            description="Water friends' crops"
            checked={config.auto_help_water}
            onChange={() => toggle("auto_help_water")}
          />
          <ToggleRow
            label="Auto Help Weed"
            description="Remove weeds from friends"
            checked={config.auto_help_weed}
            onChange={() => toggle("auto_help_weed")}
          />
          <ToggleRow
            label="Auto Help Insecticide"
            description="Remove insects from friends"
            checked={config.auto_help_insecticide}
            onChange={() => toggle("auto_help_insecticide")}
          />
        </div>
      </Card>

      <Card title="Preferences">
        <div className="space-y-3">
          <div>
            <label className="text-sm font-medium">Preferred Seed ID</label>
            <p className="text-xs text-on-surface-muted mb-1.5">
              Seed to auto-plant (leave empty for default)
            </p>
            <input
              type="number"
              value={config.preferred_seed_id ?? ""}
              onChange={(e) => {
                const val = e.target.value;
                setConfig((prev) => ({
                  ...prev,
                  preferred_seed_id: val ? parseInt(val, 10) : null,
                }));
                setSaved(false);
              }}
              placeholder="e.g. 10001"
              className="w-full rounded-lg border border-border bg-surface-dim px-3 py-2 text-sm outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
            />
          </div>
        </div>
      </Card>
    </div>
  );
}
