import { useState } from "react";
import { Sparkles, Copy, Loader2, Check } from "lucide-react";
import { useAiStatus } from "@/hooks/use-ai";
import { useAcceptEnrichmentJob, useIncidentEnrichment, useRunIncidentEnrichment } from "@/hooks/use-enrichments";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { toast } from "@/components/ui/use-toast";

interface AiSummaryPanelProps {
  incidentId: string;
}

export function AiSummaryPanel({
  incidentId,
}: AiSummaryPanelProps) {
  const { data: aiStatus } = useAiStatus();
  const { data: enrichment } = useIncidentEnrichment(incidentId);
  const runEnrichment = useRunIncidentEnrichment();
  const accept = useAcceptEnrichmentJob();
  const [summaryText, setSummaryText] = useState<string | null>(null);
  const [stakeholderText, setStakeholderText] = useState<string | null>(null);
  const [summaryJobId, setSummaryJobId] = useState<string | null>(null);
  const [stakeholderJobId, setStakeholderJobId] = useState<string | null>(null);

  if (!aiStatus?.available) return null;

  const handleSummarize = async () => {
    try {
      const job = await runEnrichment.mutateAsync({
        job_type: "incident_executive_summary",
        incident_id: incidentId,
      });
      if (job.status !== "succeeded") {
        throw new Error(job.error || "Enrichment job failed");
      }
      const parsed = JSON.parse(job.output_json) as { summary?: string };
      setSummaryText(parsed.summary ?? "");
      setSummaryJobId(job.id);
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
      const job = await runEnrichment.mutateAsync({
        job_type: "stakeholder_update",
        incident_id: incidentId,
      });
      if (job.status !== "succeeded") {
        throw new Error(job.error || "Enrichment job failed");
      }
      const parsed = JSON.parse(job.output_json) as { content?: string };
      setStakeholderText(parsed.content ?? "");
      setStakeholderJobId(job.id);
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
        {enrichment?.executive_summary ? (
          <div className="rounded-md border bg-muted/50 p-3">
            <div className="mb-1 flex items-center justify-between">
              <span className="text-xs font-medium text-muted-foreground">
                Saved Executive Summary
              </span>
              <Button
                size="sm"
                variant="ghost"
                className="h-6 px-2"
                onClick={() => copyToClipboard(enrichment.executive_summary)}
              >
                <Copy className="h-3 w-3" />
              </Button>
            </div>
            <p className="whitespace-pre-wrap text-sm">{enrichment.executive_summary}</p>
          </div>
        ) : null}

        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={handleSummarize}
            disabled={runEnrichment.isPending}
          >
            {runEnrichment.isPending ? (
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
            disabled={runEnrichment.isPending}
          >
            {runEnrichment.isPending ? (
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
              <div className="flex items-center gap-1">
                {summaryJobId ? (
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-6 px-2"
                    onClick={async () => {
                      try {
                        await accept.mutateAsync(summaryJobId);
                        toast({ title: "Saved to incident enrichments" });
                      } catch (err) {
                        toast({ title: "Save failed", description: String(err), variant: "destructive" });
                      }
                    }}
                    disabled={accept.isPending}
                    title="Accept and save"
                  >
                    <Check className="h-3 w-3" />
                  </Button>
                ) : null}
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-6 px-2"
                  onClick={() => copyToClipboard(summaryText)}
                  title="Copy"
                >
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
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
              <div className="flex items-center gap-1">
                {stakeholderJobId ? (
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-6 px-2"
                    onClick={async () => {
                      try {
                        await accept.mutateAsync(stakeholderJobId);
                        toast({ title: "Saved as stakeholder update" });
                      } catch (err) {
                        toast({ title: "Save failed", description: String(err), variant: "destructive" });
                      }
                    }}
                    disabled={accept.isPending}
                    title="Accept and save"
                  >
                    <Check className="h-3 w-3" />
                  </Button>
                ) : null}
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-6 px-2"
                  onClick={() => copyToClipboard(stakeholderText)}
                  title="Copy"
                >
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
            </div>
            <p className="whitespace-pre-wrap text-sm">{stakeholderText}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
