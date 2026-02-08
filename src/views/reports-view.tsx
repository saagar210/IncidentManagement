import { useState, useCallback } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { FileText, Download, Loader2, Eye, CheckSquare, Square } from "lucide-react";
import { useQuarters } from "@/hooks/use-quarters";
import { useGenerateReport, useSaveReport, useDiscussionPoints } from "@/hooks/use-reports";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/components/ui/use-toast";
import type { ReportSections, DiscussionPoint } from "@/types/reports";

const DEFAULT_SECTIONS: ReportSections = {
  executive_summary: true,
  metrics_overview: true,
  incident_timeline: true,
  incident_breakdowns: true,
  service_reliability: true,
  qoq_comparison: true,
  discussion_points: true,
  action_items: true,
};

const SECTION_LABELS: Record<keyof ReportSections, string> = {
  executive_summary: "Executive Summary",
  metrics_overview: "Metrics Overview",
  incident_timeline: "Incident Timeline",
  incident_breakdowns: "Critical Incident Breakdowns",
  service_reliability: "Service Reliability",
  qoq_comparison: "Quarter-over-Quarter Comparison",
  discussion_points: "Discussion Points",
  action_items: "Action Items",
};

function severityColor(severity: string): string {
  switch (severity) {
    case "critical":
      return "bg-red-500/10 text-red-500 border-red-500/20";
    case "high":
      return "bg-orange-500/10 text-orange-500 border-orange-500/20";
    case "medium":
      return "bg-yellow-500/10 text-yellow-600 border-yellow-500/20";
    case "low":
      return "bg-green-500/10 text-green-600 border-green-500/20";
    default:
      return "bg-zinc-500/10 text-zinc-500 border-zinc-500/20";
  }
}

export function ReportsView() {
  const { data: quarters, isLoading: quartersLoading } = useQuarters();
  const generateReport = useGenerateReport();
  const saveReport = useSaveReport();

  const [selectedQuarterId, setSelectedQuarterId] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [introduction, setIntroduction] = useState("");
  const [sections, setSections] = useState<ReportSections>(DEFAULT_SECTIONS);
  const [showDiscussionPreview, setShowDiscussionPreview] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);

  const { data: discussionPoints, isLoading: discussionLoading } =
    useDiscussionPoints(showDiscussionPreview ? selectedQuarterId : null);

  // Build default title from quarter selection
  const selectedQuarter = quarters?.find((q) => q.id === selectedQuarterId);
  const effectiveTitle = title || (selectedQuarter ? `${selectedQuarter.label} Incident Review` : "Incident Review Report");

  const toggleSection = useCallback((key: keyof ReportSections) => {
    setSections((prev) => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const toggleAll = useCallback((checked: boolean) => {
    const updated = { ...sections };
    for (const key of Object.keys(updated) as (keyof ReportSections)[]) {
      updated[key] = checked;
    }
    setSections(updated);
  }, [sections]);

  const allChecked = Object.values(sections).every(Boolean);
  const noneChecked = Object.values(sections).every((v) => !v);

  const handleGenerate = useCallback(async () => {
    if (!selectedQuarterId) {
      toast({
        title: "Select a quarter",
        description: "Please select a quarter before generating a report.",
        variant: "destructive",
      });
      return;
    }

    setIsGenerating(true);

    try {
      // Generate the report (returns temp file path)
      const tempPath = await generateReport.mutateAsync({
        quarter_id: selectedQuarterId,
        fiscal_year: selectedQuarter?.fiscal_year ?? null,
        title: effectiveTitle,
        introduction,
        sections,
        chart_images: {},
      });

      // Prompt user for save location
      const savePath = await save({
        defaultPath: `${effectiveTitle.replace(/[^a-zA-Z0-9]/g, "_")}.docx`,
        filters: [{ name: "Word Document", extensions: ["docx"] }],
      });

      if (savePath) {
        await saveReport.mutateAsync({ tempPath, savePath });
        toast({
          title: "Report saved",
          description: `Report saved to ${savePath}`,
        });
      } else {
        // User cancelled, clean up temp file by saving to nowhere
        toast({
          title: "Report generated",
          description: "Report was generated but save was cancelled. The temporary file will be cleaned up.",
        });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast({
        title: "Report generation failed",
        description: message,
        variant: "destructive",
      });
    } finally {
      setIsGenerating(false);
    }
  }, [selectedQuarterId, selectedQuarter, effectiveTitle, introduction, sections, generateReport, saveReport]);

  const handlePreviewDiscussion = useCallback(() => {
    if (!selectedQuarterId) {
      toast({
        title: "Select a quarter",
        description: "Please select a quarter to preview discussion points.",
        variant: "destructive",
      });
      return;
    }
    setShowDiscussionPreview(true);
  }, [selectedQuarterId]);

  return (
    <div className="p-6 max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold flex items-center gap-2">
            <FileText className="h-6 w-6" />
            Report Generator
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Generate quarterly incident review reports as DOCX files.
          </p>
        </div>
      </div>

      {/* Quarter Selection */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Report Configuration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>Quarter</Label>
              {quartersLoading ? (
                <div className="h-10 bg-muted animate-pulse rounded" />
              ) : (
                <Select
                  value={selectedQuarterId ?? ""}
                  onChange={(e) => {
                    setSelectedQuarterId(e.target.value || null);
                    setShowDiscussionPreview(false);
                  }}
                >
                  <option value="">Select a quarter...</option>
                  {quarters?.map((q) => (
                    <option key={q.id} value={q.id}>
                      {q.label} (FY{q.fiscal_year})
                    </option>
                  ))}
                </Select>
              )}
            </div>

            <div className="space-y-2">
              <Label>Report Title</Label>
              <Input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder={effectiveTitle}
              />
            </div>
          </div>

          <div className="space-y-2">
            <Label>Custom Introduction (optional)</Label>
            <Textarea
              value={introduction}
              onChange={(e) => setIntroduction(e.target.value)}
              placeholder="Add a custom introduction paragraph for the executive summary..."
              rows={3}
            />
          </div>
        </CardContent>
      </Card>

      {/* Section Selection */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">Report Sections</CardTitle>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => toggleAll(!allChecked)}
            >
              {allChecked ? "Deselect All" : "Select All"}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-3">
            {(Object.keys(SECTION_LABELS) as (keyof ReportSections)[]).map(
              (key) => (
                <button
                  key={key}
                  type="button"
                  className="flex items-center gap-3 p-3 rounded-lg border hover:bg-muted/50 transition-colors text-left"
                  onClick={() => toggleSection(key)}
                >
                  {sections[key] ? (
                    <CheckSquare className="h-5 w-5 text-primary shrink-0" />
                  ) : (
                    <Square className="h-5 w-5 text-muted-foreground shrink-0" />
                  )}
                  <span className="text-sm">{SECTION_LABELS[key]}</span>
                </button>
              )
            )}
          </div>
        </CardContent>
      </Card>

      {/* Actions */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex items-center gap-3">
            <Button
              onClick={handleGenerate}
              disabled={isGenerating || !selectedQuarterId || noneChecked}
              className="gap-2"
            >
              {isGenerating ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Generating...
                </>
              ) : (
                <>
                  <Download className="h-4 w-4" />
                  Generate Report
                </>
              )}
            </Button>

            <Button
              variant="outline"
              onClick={handlePreviewDiscussion}
              disabled={!selectedQuarterId || discussionLoading}
              className="gap-2"
            >
              {discussionLoading ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Eye className="h-4 w-4" />
              )}
              Preview Discussion Points
            </Button>
          </div>

          {!selectedQuarterId && (
            <p className="text-sm text-muted-foreground mt-2">
              Select a quarter above to enable report generation.
            </p>
          )}
        </CardContent>
      </Card>

      {/* Discussion Points Preview */}
      {showDiscussionPreview && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Discussion Points Preview</CardTitle>
          </CardHeader>
          <CardContent>
            {discussionLoading ? (
              <div className="space-y-3">
                {[1, 2, 3].map((i) => (
                  <div key={i} className="h-12 bg-muted animate-pulse rounded" />
                ))}
              </div>
            ) : discussionPoints && discussionPoints.length > 0 ? (
              <div className="space-y-3">
                {discussionPoints.map((point, idx) => (
                  <DiscussionPointCard key={idx} point={point} index={idx + 1} />
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                No automatic discussion points generated for this quarter.
              </p>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function DiscussionPointCard({
  point,
  index,
}: {
  point: DiscussionPoint;
  index: number;
}) {
  return (
    <div className="flex gap-3 p-3 rounded-lg border">
      <span className="text-sm font-medium text-muted-foreground shrink-0 mt-0.5">
        {index}.
      </span>
      <div className="space-y-1.5 min-w-0">
        <div className="flex items-center gap-2">
          <Badge className={severityColor(point.severity)}>
            {point.severity.toUpperCase()}
          </Badge>
        </div>
        <p className="text-sm">{point.text}</p>
        <p className="text-xs text-muted-foreground">{point.trigger}</p>
      </div>
    </div>
  );
}
