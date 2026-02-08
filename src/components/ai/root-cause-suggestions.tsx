import { useState } from "react";
import { Sparkles, Loader2, Copy } from "lucide-react";
import { useMutation } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import { useAiStatus } from "@/hooks/use-ai";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { toast } from "@/components/ui/use-toast";

interface RootCauseSuggestionsProps {
  title: string;
  severity: string;
  service: string;
  symptoms: string;
  timeline: string;
}

export function RootCauseSuggestions({
  title,
  severity,
  service,
  symptoms,
  timeline,
}: RootCauseSuggestionsProps) {
  const { data: aiStatus } = useAiStatus();
  const [suggestions, setSuggestions] = useState<string | null>(null);

  const mutation = useMutation({
    mutationFn: () =>
      tauriInvoke<string>("ai_suggest_root_causes", {
        title,
        severity,
        service,
        symptoms,
        timeline,
      }),
  });

  if (!aiStatus?.available) return null;

  const handleSuggest = async () => {
    try {
      const result = await mutation.mutateAsync();
      setSuggestions(result);
    } catch (err) {
      toast({
        title: "AI suggestions failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const copyToClipboard = async (text: string) => {
    await navigator.clipboard.writeText(text);
    toast({ title: "Copied to clipboard" });
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-base">
          <Sparkles className="h-4 w-4 text-orange-500" />
          Root Cause Analysis
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <Button
          size="sm"
          variant="outline"
          onClick={handleSuggest}
          disabled={mutation.isPending}
        >
          {mutation.isPending ? (
            <Loader2 className="mr-1 h-3 w-3 animate-spin" />
          ) : (
            <Sparkles className="mr-1 h-3 w-3" />
          )}
          {mutation.isPending ? "Analyzing..." : "Suggest Root Causes"}
        </Button>

        {suggestions && (
          <div className="rounded-md border bg-muted/50 p-3">
            <div className="mb-1 flex items-center justify-between">
              <span className="text-xs font-medium text-muted-foreground">
                AI Suggestions
              </span>
              <Button
                size="sm"
                variant="ghost"
                className="h-6 px-2"
                onClick={() => copyToClipboard(suggestions)}
              >
                <Copy className="h-3 w-3" />
              </Button>
            </div>
            <p className="whitespace-pre-wrap text-sm">{suggestions}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
