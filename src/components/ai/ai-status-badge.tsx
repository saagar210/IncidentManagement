import { Bot, BotOff } from "lucide-react";
import { useAiStatus, useCheckAiHealth } from "@/hooks/use-ai";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export function AiStatusBadge() {
  const { data: status } = useAiStatus();
  const healthCheck = useCheckAiHealth();

  const available = status?.available ?? false;

  return (
    <Badge
      variant="outline"
      className={cn(
        "cursor-pointer gap-1 text-xs",
        available
          ? "bg-green-500/10 text-green-600 border-green-500/20"
          : "bg-muted text-muted-foreground"
      )}
      onClick={() => healthCheck.mutate()}
    >
      {available ? (
        <Bot className="h-3 w-3" />
      ) : (
        <BotOff className="h-3 w-3" />
      )}
      AI {available ? "Ready" : "Offline"}
    </Badge>
  );
}
