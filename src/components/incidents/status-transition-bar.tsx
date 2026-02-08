import { ArrowRight, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  STATUS_TRANSITIONS,
  STATUS_COLORS,
  type StatusOption,
} from "@/lib/constants";

interface StatusTransitionBarProps {
  currentStatus: string;
  reopenCount: number;
  onTransition: (newStatus: string) => void;
  disabled?: boolean;
}

export function StatusTransitionBar({
  currentStatus,
  reopenCount,
  onTransition,
  disabled,
}: StatusTransitionBarProps) {
  const allowed = STATUS_TRANSITIONS[currentStatus as StatusOption] ?? [];
  const isReopen = (target: string) =>
    target === "Active" &&
    (currentStatus === "Resolved" || currentStatus === "Post-Mortem");

  return (
    <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/30 p-3">
      <div className="flex items-center gap-2">
        <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
          Status
        </span>
        <Badge
          className={`${STATUS_COLORS[currentStatus as StatusOption] ?? ""} border`}
        >
          {currentStatus}
        </Badge>
        {reopenCount > 0 && (
          <Badge variant="outline" className="text-[10px]">
            <RotateCcw className="mr-1 h-2.5 w-2.5" />
            Reopened {reopenCount}x
          </Badge>
        )}
      </div>

      {allowed.length > 0 && (
        <>
          <ArrowRight className="h-3.5 w-3.5 text-muted-foreground" />
          <div className="flex flex-wrap gap-1.5">
            {allowed.map((target) => (
              <Button
                key={target}
                type="button"
                size="sm"
                variant={isReopen(target) ? "destructive" : "outline"}
                className="h-7 text-xs"
                disabled={disabled}
                onClick={() => onTransition(target)}
              >
                {isReopen(target) && (
                  <RotateCcw className="mr-1 h-3 w-3" />
                )}
                {target}
              </Button>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
