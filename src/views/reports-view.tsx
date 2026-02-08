import { useState, useCallback } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { format } from "date-fns";
import { FileText, Download, Loader2, Eye, CheckSquare, Square, History, Trash2, Sparkles } from "lucide-react";
import { useQuarters } from "@/hooks/use-quarters";
import {
  useGenerateReport,
  useSaveReport,
  useDiscussionPoints,
  useReportHistory,
  useDeleteReportHistory,
  useGenerateNarrative,
} from "@/hooks/use-reports";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { toast } from "@/components/ui/use-toast";
import type { ReportSections, ReportFormat, DiscussionPoint, ReportHistoryEntry } from "@/types/reports";

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
  const narrativeMutation = useGenerateNarrative();
  const { data: reportHistory } = useReportHistory();
  const deleteHistory = useDeleteReportHistory();

  const [selectedQuarterId, setSelectedQuarterId] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [introduction, setIntroduction] = useState("");
  const [sections, setSections] = useState<ReportSections>(DEFAULT_SECTIONS);
  const [reportFormat, setReportFormat] = useState<ReportFormat>("docx");
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
    setSections((prev) => {
      const updated = { ...prev };
      for (const key of Object.keys(updated) as (keyof ReportSections)[]) {
        updated[key] = checked;
      }
      return updated;
    });
  }, []);

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
      const ext = reportFormat === "pdf" ? "pdf" : "docx";
      const filterName = reportFormat === "pdf" ? "PDF Document" : "Word Document";

      // Generate the report (returns temp file path)
      const tempPath = await generateReport.mutateAsync({
        quarter_id: selectedQuarterId,
        fiscal_year: selectedQuarter?.fiscal_year ?? null,
        title: effectiveTitle,
        introduction,
        sections,
        chart_images: {},
        format: reportFormat,
      });

      // Prompt user for save location
      const savePath = await save({
        defaultPath: `${effectiveTitle.replace(/[^a-zA-Z0-9]/g, "_")}.${ext}`,
        filters: [{ name: filterName, extensions: [ext] }],
      });

      if (savePath) {
        await saveReport.mutateAsync({
          tempPath,
          savePath,
          title: effectiveTitle,
          quarterId: selectedQuarterId,
        });
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
            Generate quarterly incident review reports as DOCX or PDF files.
          </p>
        </div>
      </div>

      {/* Quarter Selection */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Report Configuration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-3 gap-4">
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

            <div className="space-y-2">
              <Label>Format</Label>
              <Select
                value={reportFormat}
                onChange={(e) => setReportFormat(e.target.value as ReportFormat)}
              >
                <option value="docx">Word Document (.docx)</option>
                <option value="pdf">PDF Document (.pdf)</option>
              </Select>
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>Custom Introduction (optional)</Label>
              <Button
                variant="ghost"
                size="sm"
                disabled={!selectedQuarterId || narrativeMutation.isPending}
                onClick={async () => {
                  if (!selectedQuarterId) return;
                  try {
                    const text = await narrativeMutation.mutateAsync(selectedQuarterId);
                    setIntroduction(text);
                  } catch (err) {
                    toast({
                      title: "Narrative generation failed",
                      description: String(err),
                      variant: "destructive",
                    });
                  }
                }}
                className="gap-1 text-xs"
              >
                {narrativeMutation.isPending ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <Sparkles className="h-3 w-3" />
                )}
                Auto-generate
              </Button>
            </div>
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

      {/* Report History */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <History className="h-5 w-5" />
            Report History
          </CardTitle>
        </CardHeader>
        <CardContent>
          {reportHistory && reportHistory.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Title</TableHead>
                  <TableHead>Format</TableHead>
                  <TableHead>Generated</TableHead>
                  <TableHead>Size</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {reportHistory.map((entry: ReportHistoryEntry) => (
                  <TableRow key={entry.id}>
                    <TableCell className="font-medium max-w-[200px] truncate">
                      {entry.title}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline">
                        {entry.format.toUpperCase()}
                      </Badge>
                    </TableCell>
                    <TableCell className="whitespace-nowrap text-sm">
                      {formatHistoryDate(entry.generated_at)}
                    </TableCell>
                    <TableCell className="text-sm">
                      {entry.file_size_bytes !== null
                        ? formatBytes(entry.file_size_bytes)
                        : "--"}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        size="icon"
                        variant="ghost"
                        onClick={() => {
                          const confirmed = window.confirm(
                            `Remove "${entry.title}" from history?`
                          );
                          if (confirmed) deleteHistory.mutate(entry.id);
                        }}
                        disabled={deleteHistory.isPending}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-sm text-muted-foreground">
              No reports generated yet. Generate your first report above.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function formatHistoryDate(dateStr: string): string {
  try {
    return format(new Date(dateStr), "MMM d, yyyy HH:mm");
  } catch {
    return dateStr;
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
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
