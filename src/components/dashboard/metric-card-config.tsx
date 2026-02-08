import { Settings2 } from "lucide-react";
import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import type { DashboardCardConfig } from "@/hooks/use-dashboard";

interface MetricCardConfigProps {
  config: DashboardCardConfig;
  onUpdate: (config: DashboardCardConfig) => void;
}

const CARD_LABELS: Record<keyof DashboardCardConfig, string> = {
  mttr: "MTTR",
  mtta: "MTTA",
  recurrence_rate: "Recurrence Rate",
  avg_tickets: "Avg Tickets",
  by_severity: "By Severity Chart",
  by_service: "By Service Chart",
  heatmap: "Incident Heatmap",
  hour_histogram: "Time of Day",
  trends: "Quarterly Trends",
  timeline: "Timeline View",
};

export function MetricCardConfig({ config, onUpdate }: MetricCardConfigProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [open]);

  const toggle = (key: keyof DashboardCardConfig) => {
    onUpdate({ ...config, [key]: !config[key] });
  };

  return (
    <div className="relative" ref={ref}>
      <Button
        variant="ghost"
        size="icon"
        onClick={() => setOpen(!open)}
        title="Configure dashboard cards"
      >
        <Settings2 className="h-4 w-4" />
      </Button>
      {open && (
        <div className="absolute right-0 top-full z-50 mt-1 w-56 rounded-md border bg-popover p-2 shadow-md">
          <p className="mb-2 text-xs font-medium text-muted-foreground">
            Show/Hide Cards
          </p>
          {(Object.keys(CARD_LABELS) as (keyof DashboardCardConfig)[]).map(
            (key) => (
              <label
                key={key}
                className="flex cursor-pointer items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-muted"
              >
                <input
                  type="checkbox"
                  checked={config[key]}
                  onChange={() => toggle(key)}
                  className="h-3.5 w-3.5 rounded border-input"
                />
                {CARD_LABELS[key]}
              </label>
            )
          )}
        </div>
      )}
    </div>
  );
}
