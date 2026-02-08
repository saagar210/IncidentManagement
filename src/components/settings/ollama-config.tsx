import { Bot, BotOff, RefreshCw } from "lucide-react";
import { useAiStatus, useCheckAiHealth } from "@/hooks/use-ai";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export function OllamaConfig() {
  const { data: status } = useAiStatus();
  const healthCheck = useCheckAiHealth();

  const available = status?.available ?? false;

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-lg font-semibold">AI Integration (Ollama)</h2>
        <p className="text-sm text-muted-foreground">
          AI features are powered by Ollama running locally. All data stays on
          your machine.
        </p>
      </div>

      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2 text-base">
              {available ? (
                <Bot className="h-4 w-4 text-green-500" />
              ) : (
                <BotOff className="h-4 w-4 text-muted-foreground" />
              )}
              Connection Status
            </CardTitle>
            <Badge
              variant="outline"
              className={
                available
                  ? "bg-green-500/10 text-green-600 border-green-500/20"
                  : "bg-red-500/10 text-red-600 border-red-500/20"
              }
            >
              {available ? "Connected" : "Offline"}
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="rounded-md border bg-muted/50 p-3 text-sm">
            <p className="font-medium">Endpoint</p>
            <code className="text-xs text-muted-foreground">
              http://localhost:11434
            </code>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-md border bg-muted/50 p-3 text-sm">
              <p className="font-medium">Primary Model</p>
              <code className="text-xs text-muted-foreground">
                qwen3:30b-a3b
              </code>
              <p className="mt-1 text-xs text-muted-foreground">
                Summaries, post-mortems, stakeholder drafts
              </p>
            </div>
            <div className="rounded-md border bg-muted/50 p-3 text-sm">
              <p className="font-medium">Fast Model</p>
              <code className="text-xs text-muted-foreground">qwen3:4b</code>
              <p className="mt-1 text-xs text-muted-foreground">
                Real-time suggestions (future)
              </p>
            </div>
          </div>

          <Button
            variant="outline"
            size="sm"
            onClick={() => healthCheck.mutate()}
            disabled={healthCheck.isPending}
          >
            <RefreshCw
              className={`mr-1 h-3 w-3 ${healthCheck.isPending ? "animate-spin" : ""}`}
            />
            {healthCheck.isPending ? "Checking..." : "Test Connection"}
          </Button>
        </CardContent>
      </Card>

      {!available && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Setup Instructions</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <p>To enable AI features, install and start Ollama:</p>
            <ol className="list-inside list-decimal space-y-2 text-muted-foreground">
              <li>
                Install Ollama from{" "}
                <code className="rounded bg-muted px-1 text-xs">
                  https://ollama.com
                </code>
              </li>
              <li>
                Pull the primary model:{" "}
                <code className="rounded bg-muted px-1 text-xs">
                  ollama pull qwen3:30b-a3b
                </code>
              </li>
              <li>
                (Optional) Pull the fast model:{" "}
                <code className="rounded bg-muted px-1 text-xs">
                  ollama pull qwen3:4b
                </code>
              </li>
              <li>
                Ensure Ollama is running, then click &ldquo;Test
                Connection&rdquo; above
              </li>
            </ol>
            <p className="text-xs text-muted-foreground">
              The primary model requires ~18-20GB RAM. The app works fully
              without AI — all AI features gracefully degrade.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
