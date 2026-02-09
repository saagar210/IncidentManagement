import { useState, useEffect, useRef, useCallback } from "react";
import { FileText, Sparkles, Loader2, Save, Plus } from "lucide-react";
import {
  usePostmortemByIncident,
  useCreatePostmortem,
  useUpdatePostmortem,
  usePostmortemTemplates,
  usePostmortemReadiness,
} from "@/hooks/use-postmortems";
import { usePirBriefMarkdown, useGeneratePirBriefFile } from "@/hooks/use-pir-review";
import { useSaveReport } from "@/hooks/use-reports";
import { useAiPostmortemDraft, useAiStatus } from "@/hooks/use-ai";
import { useContributingFactors } from "@/hooks/use-postmortems";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "@/components/ui/use-toast";
import { save } from "@tauri-apps/plugin-dialog";

interface PostmortemEditorProps {
  incidentId: string;
  title: string;
  severity: string;
  service: string;
  rootCause: string;
  resolution: string;
  lessons: string;
  onNavigateToTab?: (
    tab: "details" | "analysis" | "actions" | "postmortem" | "activity"
  ) => void;
}

const PM_STATUS_COLORS: Record<string, string> = {
  draft: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  review: "bg-blue-500/10 text-blue-600 border-blue-500/20",
  final: "bg-green-500/10 text-green-600 border-green-500/20",
};

export function PostmortemEditor({
  incidentId,
  title,
  severity,
  service,
  rootCause,
  resolution,
  lessons,
  onNavigateToTab,
}: PostmortemEditorProps) {
  const { data: existingPm } = usePostmortemByIncident(incidentId);
  const { data: templates } = usePostmortemTemplates();
  const { data: factors } = useContributingFactors(incidentId);
  const readiness = usePostmortemReadiness(incidentId);
  const { data: aiStatus } = useAiStatus();
  const createPm = useCreatePostmortem();
  const updatePm = useUpdatePostmortem();
  const aiDraft = useAiPostmortemDraft();
  const pirBrief = usePirBriefMarkdown();
  const generatePirFile = useGeneratePirBriefFile();
  const saveReport = useSaveReport();

  const [content, setContent] = useState("");
  const [pmStatus, setPmStatus] = useState<"draft" | "review" | "final">(
    "draft"
  );
  const [noActionItemsJustified, setNoActionItemsJustified] = useState(false);
  const [noActionItemsJustification, setNoActionItemsJustification] =
    useState("");
  const contentRef = useRef(content);
  contentRef.current = content;

  useEffect(() => {
    if (existingPm) {
      // Parse content if it's JSON with a "markdown" field, otherwise use raw
      try {
        const parsed = JSON.parse(existingPm.content);
        setContent(parsed.markdown ?? existingPm.content);
      } catch {
        setContent(existingPm.content === "{}" ? "" : existingPm.content);
      }
      setPmStatus(
        existingPm.status === "final"
          ? "final"
          : existingPm.status === "review"
            ? "review"
            : "draft"
      );
      setNoActionItemsJustified(!!existingPm.no_action_items_justified);
      setNoActionItemsJustification(
        existingPm.no_action_items_justification ?? ""
      );
    }
  }, [existingPm]);

  const handleCreate = async (templateId?: string) => {
    try {
      await createPm.mutateAsync({
        incident_id: incidentId,
        template_id: templateId,
      });
      toast({ title: "Post-mortem created" });
    } catch (err) {
      toast({
        title: "Failed to create post-mortem",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleSave = useCallback(async () => {
    if (!existingPm) return;
    try {
      await updatePm.mutateAsync({
        id: existingPm.id,
        req: {
          content: JSON.stringify({ markdown: contentRef.current }),
          status: pmStatus,
          no_action_items_justified: noActionItemsJustified,
          no_action_items_justification: noActionItemsJustification,
        },
      });
      readiness.refetch();
      toast({ title: "Post-mortem saved" });
    } catch (err) {
      toast({
        title: "Failed to save",
        description: String(err),
        variant: "destructive",
      });
    }
  }, [
    existingPm,
    noActionItemsJustification,
    noActionItemsJustified,
    pmStatus,
    readiness,
    updatePm,
  ]);

  const handleFinalize = useCallback(async () => {
    if (!existingPm) return;
    try {
      // Step 1: persist current content/exception fields without changing to final.
      await updatePm.mutateAsync({
        id: existingPm.id,
        req: {
          content: JSON.stringify({ markdown: contentRef.current }),
          status: pmStatus,
          no_action_items_justified: noActionItemsJustified,
          no_action_items_justification: noActionItemsJustification,
        },
      });

      // Step 2: re-check readiness against the persisted state.
      const r = await readiness.refetch();
      const data = r.data;
      if (!data || !data.can_finalize) {
        const first = data?.missing?.[0];
        toast({
          title: "Not ready to finalize",
          description: "Complete the missing items in the checklist.",
          variant: "destructive",
        });
        if (first?.destination) onNavigateToTab?.(first.destination);
        return;
      }

      // Step 3: finalize.
      await updatePm.mutateAsync({
        id: existingPm.id,
        req: {
          status: "final",
        },
      });
      readiness.refetch();
      toast({ title: "Post-mortem finalized" });
    } catch (err) {
      toast({
        title: "Failed to finalize",
        description: String(err),
        variant: "destructive",
      });
    }
  }, [
    existingPm,
    noActionItemsJustification,
    noActionItemsJustified,
    onNavigateToTab,
    pmStatus,
    readiness,
    updatePm,
  ]);

  const handleAiDraft = async () => {
    try {
      const factorDescriptions =
        factors?.map((f) => `[${f.category}] ${f.description}`) ?? [];
      const result = await aiDraft.mutateAsync({
        title,
        severity,
        service,
        rootCause,
        resolution,
        lessons,
        contributingFactors: factorDescriptions,
      });
      setContent(result);
      toast({ title: "AI draft generated" });
    } catch (err) {
      toast({
        title: "AI draft failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleCopyPirBrief = useCallback(async () => {
    try {
      const brief = await pirBrief.mutateAsync(incidentId);
      await navigator.clipboard.writeText(brief.markdown);
      toast({ title: "PIR brief copied" });
    } catch (err) {
      toast({
        title: "Failed to copy PIR brief",
        description: String(err),
        variant: "destructive",
      });
    }
  }, [incidentId, pirBrief]);

  const handleExportPirBrief = useCallback(
    async (format: "docx" | "pdf") => {
      try {
        const tempPath = await generatePirFile.mutateAsync({ incidentId, format });
        const ext = format;
        const filterName = format === "pdf" ? "PDF Document" : "Word Document";
        const safeTitle = title.replace(/[^a-zA-Z0-9]/g, "_").slice(0, 80) || "pir_brief";

        const savePath = await save({
          defaultPath: `${safeTitle}_PIR_Brief.${ext}`,
          filters: [{ name: filterName, extensions: [ext] }],
        });

        if (!savePath) return;

        await saveReport.mutateAsync({
          tempPath,
          savePath,
          title: `PIR Brief - ${title}`,
          quarterId: null,
          configJson: JSON.stringify({ kind: "pir_brief", incident_id: incidentId, format }),
        });
        toast({ title: "PIR brief exported", description: `Saved to ${savePath}` });
      } catch (err) {
        toast({
          title: "Failed to export PIR brief",
          description: String(err),
          variant: "destructive",
        });
      }
    },
    [generatePirFile, incidentId, saveReport, title]
  );

  // No PM yet — show create button
  if (!existingPm) {
    return (
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <FileText className="h-4 w-4" />
            Post-Mortem
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            No post-mortem has been created for this incident.
          </p>
          <div className="flex flex-wrap gap-2">
            {templates?.map((t) => (
              <Button
                key={t.id}
                size="sm"
                variant="outline"
                onClick={() => handleCreate(t.id)}
              >
                <FileText className="mr-1 h-3 w-3" />
                {t.name}
              </Button>
            ))}
            <Button size="sm" onClick={() => handleCreate()}>
              <Plus className="mr-1 h-3 w-3" />
              Blank Post-Mortem
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <FileText className="h-4 w-4" />
            Post-Mortem
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge
              variant="outline"
              className={PM_STATUS_COLORS[pmStatus] ?? ""}
            >
              {pmStatus}
            </Badge>
            <div className="flex gap-1">
              <Button
                size="sm"
                variant={pmStatus === "draft" ? "default" : "outline"}
                onClick={() => setPmStatus("draft")}
              >
                Draft
              </Button>
              <Button
                size="sm"
                variant={pmStatus === "review" ? "default" : "outline"}
                onClick={() => setPmStatus("review")}
              >
                Review
              </Button>
            </div>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <Card className="border-dashed">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Finalize Checklist</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            {readiness.data ? (
              readiness.data.can_finalize ? (
                <div className="text-muted-foreground">
                  Ready to finalize.
                </div>
              ) : (
                <div className="space-y-1">
                  <div className="text-muted-foreground">
                    Missing items:
                  </div>
                  <ul className="list-disc pl-5">
                    {readiness.data.missing.map((m) => (
                      <li key={m.code}>
                        <button
                          type="button"
                          className="text-left underline-offset-2 hover:underline"
                          onClick={() => {
                            if (m.destination === "actions") {
                              onNavigateToTab?.("actions");
                              return;
                            }

                            if (m.code === "CONTRIBUTING_FACTORS") {
                              document
                                .getElementById("pir-contributing-factors")
                                ?.scrollIntoView({ behavior: "smooth", block: "start" });
                              return;
                            }

                            if (m.code === "POSTMORTEM_MARKDOWN") {
                              document
                                .getElementById("pir-postmortem-markdown")
                                ?.scrollIntoView({ behavior: "smooth", block: "start" });
                              return;
                            }

                            if (m.code === "ACTION_ITEMS_JUSTIFICATION") {
                              document
                                .getElementById("pir-action-items-exception")
                                ?.scrollIntoView({ behavior: "smooth", block: "start" });
                            }
                          }}
                        >
                          {m.label}
                        </button>
                      </li>
                    ))}
                  </ul>
                  <div className="text-muted-foreground">
                    Tip: action items are managed in the “Actions & Extras” tab.
                  </div>
                </div>
              )
            ) : (
              <div className="text-muted-foreground">
                Readiness check unavailable until a post-mortem exists.
              </div>
            )}
          </CardContent>
        </Card>

        <div id="pir-action-items-exception" className="space-y-2">
          <Label>Action Items Exception</Label>
          <div className="flex items-center gap-2">
            <input
              id="no-actions-justified"
              type="checkbox"
              checked={noActionItemsJustified}
              onChange={(e) => setNoActionItemsJustified(e.target.checked)}
            />
            <label htmlFor="no-actions-justified" className="text-sm">
              No action items are justified for this incident (requires explanation)
            </label>
          </div>
          {noActionItemsJustified && (
            <Textarea
              value={noActionItemsJustification}
              onChange={(e) => setNoActionItemsJustification(e.target.value)}
              placeholder="Explain why no action items are required (e.g., pure vendor outage with no internal mitigations available)."
            />
          )}
        </div>

        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={handleCopyPirBrief}
            disabled={pirBrief.isPending}
          >
            Copy PIR Brief
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleExportPirBrief("docx")}
            disabled={generatePirFile.isPending || saveReport.isPending}
          >
            Export DOCX
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleExportPirBrief("pdf")}
            disabled={generatePirFile.isPending || saveReport.isPending}
          >
            Export PDF
          </Button>
          {aiStatus?.available && (
            <Button
              size="sm"
              variant="outline"
              onClick={handleAiDraft}
              disabled={aiDraft.isPending}
            >
              {aiDraft.isPending ? (
                <Loader2 className="mr-1 h-3 w-3 animate-spin" />
              ) : (
                <Sparkles className="mr-1 h-3 w-3" />
              )}
              AI Draft
            </Button>
          )}
          <Button
            size="sm"
            onClick={handleSave}
            disabled={updatePm.isPending}
          >
            <Save className="mr-1 h-3 w-3" />
            Save
          </Button>
          {pmStatus !== "final" && (
            <Button
              size="sm"
              variant="secondary"
              onClick={handleFinalize}
              disabled={updatePm.isPending}
            >
              Finalize
            </Button>
          )}
        </div>

        <div id="pir-postmortem-markdown">
          <MarkdownEditor value={content} onChange={setContent} />
        </div>
      </CardContent>
    </Card>
  );
}
