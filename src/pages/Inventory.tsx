import { useState, useEffect, useCallback } from "react";
import { Package, ShoppingCart } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { useTauriEvent } from "../hooks/useTauriEvent";
import * as api from "../api";

interface BagItem {
  item_id?: number;
  name?: string;
  count?: number;
  type_name?: string;
}

export default function InventoryPage() {
  const [items, setItems] = useState<BagItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [selling, setSelling] = useState(false);

  const fetchBag = useCallback(async () => {
    setLoading(true);
    try {
      const res = (await api.getBag()) as { items?: BagItem[] };
      setItems(res?.items ?? (Array.isArray(res) ? res : []));
    } catch (e) {
      console.error("Failed to load bag:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBag();
  }, [fetchBag]);

  const handleDataChanged = useCallback(
    (scope: string) => {
      if (scope === "inventory") fetchBag();
    },
    [fetchBag]
  );
  useTauriEvent("data-changed", handleDataChanged);

  const handleSellAll = async () => {
    setSelling(true);
    try {
      await api.sellAllFruits();
      await fetchBag();
    } catch (e) {
      console.error("Sell failed:", e);
    } finally {
      setSelling(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">仓库</h1>
          <p className="text-sm text-on-surface-muted">
            共 {items.length} 件物品
          </p>
        </div>
        <Button
          variant="danger"
          size="sm"
          icon={<ShoppingCart className="size-3.5" />}
          onClick={handleSellAll}
          loading={selling}
        >
          卖出果实
        </Button>
      </div>

      {items.length === 0 && !loading ? (
        <EmptyState
          icon={<Package className="size-10" />}
          title="背包为空"
          description="请先连接游戏服务器"
        />
      ) : (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
          {items.map((item, idx) => (
            <Card key={item.item_id ?? idx}>
              <div className="space-y-1">
                <p className="text-sm font-medium truncate">
                  {item.name ?? `物品 #${item.item_id}`}
                </p>
                {item.type_name && (
                  <p className="text-xs text-on-surface-muted">
                    {item.type_name}
                  </p>
                )}
                <p className="text-lg font-bold text-primary-600">
                  x{item.count ?? 0}
                </p>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
