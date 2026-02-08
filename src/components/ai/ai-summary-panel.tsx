import { useState } from "react";
import { Sparkles, Copy, Loader2 } from "lucide-react";
import { useAiSummarize, useAiStakeholderUpdate, useAiStatus } from "@/hooks/use-ai";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { toast } from "@/components/ui/use-toast";

interface AiSummaryPanelProps {
  title: string;
  severity: string;
  status: string;
  service: string;
  impact: string;
  rootCause: string;
  resolution: string;
  notes: string;
}

export function AiSummaryPanel({
  title,
  severity,
  status,
  service,
  impact,
  rootCause,
  resolution,
  notes,
}: AiSummaryPanelProps) {
  const { data: aiStatus } = useAiStatus();
  const summarize = useAiSummarize();
  const stakeholder = useAiStakeholderUpdate();
  const [summaryText, setSummaryText] = useState<string | null>(null);
  const [stakeholderText, setStakeholderText] = useState<string | null>(null);

  if (!aiStatus?.available) return null;

  const handleSummarize = async () => {
    try {
      const result = await summarize.mutateAsync({
        title,
        severity,
        status,
        service,
        rootCause,
        resolution,
        notes,
      });
      setSummaryText(result);
    } catch (err) {
      toast({
        title: "AI Summary failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleStakeholder = async () => {
    try {
      const result = await stakeholder.mutateAsync({
        title,
        severity,
        status,
        service,
        impact,
        notes,
      });
      setStakeholderText(result);
    } catch (err) {
      toast({
        title: "AI Stakeholder update failed",
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
          <Sparkles className="h-4 w-4 text-purple-500" />
          AI Assistant
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={handleSummarize}
            disabled={summarize.isPending}
          >
            {summarize.isPending ? (
              <Loader2 className="mr-1 h-3 w-3 animate-spin" />
            ) : (
              <Sparkles className="mr-1 h-3 w-3" />
            )}
            Summarize
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={handleStakeholder}
            disabled={stakeholder.isPending}
          >
            {stakeholder.isPending ? (
              <Loader2 className="mr-1 h-3 w-3 animate-spin" />
            ) : (
              <Sparkles className="mr-1 h-3 w-3" />
            )}
            Stakeholder Draft
          </Button>
        </div>

        {summaryText && (
          <div className="rounded-md border bg-muted/50 p-3">
            <div className="mb-1 flex items-center justify-between">
              <span className="text-xs font-medium text-muted-foreground">
                Executive Summary
              </span>
              <Button
                size="sm"
                variant="ghost"
                className="h-6 px-2"
                onClick={() => copyToClipboard(summaryText)}
              >
                <Copy className="h-3 w-3" />
              </Button>
            </div>
            <p className="whitespace-pre-wrap text-sm">{summaryText}</p>
          </div>
        )}

        {stakeholderText && (
          <div className="rounded-md border bg-muted/50 p-3">
            <div className="mb-1 flex items-center justify-between">
              <span className="text-xs font-medium text-muted-foreground">
                Stakeholder Update
              </span>
              <Button
                size="sm"
                variant="ghost"
                className="h-6 px-2"
                onClick={() => copyToClipboard(stakeholderText)}
              >
                <Copy className="h-3 w-3" />
              </Button>
            </div>
            <p className="whitespace-pre-wrap text-sm">{stakeholderText}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
