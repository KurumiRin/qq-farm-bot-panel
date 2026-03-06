import { useState, useCallback, createContext, useContext, useRef } from "react";
import { CheckCircle2, XCircle, Info } from "lucide-react";

interface ToastItem {
  id: number;
  type: "success" | "error" | "info";
  message: string;
}

interface ToastContextType {
  toast: (type: ToastItem["type"], message: string) => void;
}

const ToastContext = createContext<ToastContextType>({ toast: () => {} });

export function useToast() {
  return useContext(ToastContext);
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);
  const idRef = useRef(0);

  const toast = useCallback((type: ToastItem["type"], message: string) => {
    const id = ++idRef.current;
    setItems((prev) => [...prev, { id, type, message }]);
    setTimeout(() => {
      setItems((prev) => prev.filter((t) => t.id !== id));
    }, 3000);
  }, []);

  const icons = {
    success: <CheckCircle2 className="size-4 shrink-0 text-green-500" />,
    error: <XCircle className="size-4 shrink-0 text-red-500" />,
    info: <Info className="size-4 shrink-0 text-blue-500" />,
  };

  return (
    <ToastContext value={{ toast }}>
      {children}
      <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
        {items.map((item) => (
          <div
            key={item.id}
            className="flex items-start gap-2 rounded-lg border border-border bg-surface px-3 py-2 shadow-lg text-sm animate-in slide-in-from-right duration-200"
          >
            {icons[item.type]}
            <span className="text-on-surface">{item.message}</span>
          </div>
        ))}
      </div>
    </ToastContext>
  );
}
