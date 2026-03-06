import { useState, useEffect, useCallback } from "react";
import { Package, RefreshCw, ShoppingCart } from "lucide-react";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
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
          <h1 className="text-xl font-bold">Inventory</h1>
          <p className="text-sm text-on-surface-muted">
            {items.length} items in bag
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            size="sm"
            icon={<RefreshCw className="size-3.5" />}
            onClick={fetchBag}
            loading={loading}
          >
            Refresh
          </Button>
          <Button
            variant="danger"
            size="sm"
            icon={<ShoppingCart className="size-3.5" />}
            onClick={handleSellAll}
            loading={selling}
          >
            Sell Fruits
          </Button>
        </div>
      </div>

      {items.length === 0 && !loading ? (
        <EmptyState
          icon={<Package className="size-10" />}
          title="Bag is empty"
          description="Connect to the game server to view your inventory"
        />
      ) : (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
          {items.map((item, idx) => (
            <Card key={item.item_id ?? idx}>
              <div className="space-y-1">
                <p className="text-sm font-medium truncate">
                  {item.name ?? `Item #${item.item_id}`}
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
