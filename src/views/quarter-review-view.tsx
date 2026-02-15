import { useMemo, useState, useCallback } from "react";
import { FileText, AlertTriangle, CheckCircle2, CircleAlert, ExternalLink, Lock, Unlock, Trash2 } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { useQuarters } from "@/hooks/use-quarters";
import { useDashboardData } from "@/hooks/use-metrics";
import { useMetricGlossary, useQuarterFinalizationStatus, useUpsertQuarterOverride, useDeleteQuarterOverride, useFinalizeQuarter, useUnfinalizeQuarter } from "@/hooks/use-quarter-review";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "@/components/ui/use-toast";
import { tauriInvoke } from "@/lib/tauri";

function severityBadge(sev: string) {
  if (sev === "critical") return <Badge variant="destructive">Critical</Badge>;
  if (sev === "warning") return <Badge variant="secondary">Warning</Badge>;
  return <Badge variant="secondary">{sev}</Badge>;
}

export function QuarterReviewView() {
  const navigate = useNavigate();
  const { data: quarters, isLoading: quartersLoading } = useQuarters();
  const { data: glossary } = useMetricGlossary();

  const [selectedQuarterId, setSelectedQuarterId] = useState<string | null>(null);
  const [overrideDialogOpen, setOverrideDialogOpen] = useState(false);
  const [overrideRuleKey, setOverrideRuleKey] = useState("");
  const [overrideIncidentId, setOverrideIncidentId] = useState("");
  const [overrideReason, setOverrideReason] = useState("");
  const [packetBusy, setPacketBusy] = useState(false);
  const activeQuarterId = useMemo(
    () => selectedQuarterId ?? quarters?.[0]?.id ?? null,
    [selectedQuarterId, quarters]
  );
  const selectedQuarter = useMemo(
    () => quarters?.find((q) => q.id === activeQuarterId),
    [quarters, activeQuarterId]
  );

  const { data: dashboard, isLoading: dashboardLoading } = useDashboardData(activeQuarterId);
  const { data: finalStatus, isLoading: finalLoading } = useQuarterFinalizationStatus(activeQuarterId);
  const upsertOverride = useUpsertQuarterOverride();
  const deleteOverride = useDeleteQuarterOverride();
  const finalize = useFinalizeQuarter();
  const unfinalize = useUnfinalizeQuarter();

  const readiness = finalStatus?.readiness;
  const overrides = finalStatus?.overrides ?? [];

  const status = useMemo(() => {
    if (!readiness) return "unknown";
    const hasCritical = readiness.findings.some((f) => f.severity === "critical");
    if (hasCritical) return "needs_attention";
    if (readiness.needs_attention_incidents > 0) return "warning";
    return "ready";
  }, [readiness]);

  const missingOverrides = useMemo(() => {
    if (!readiness) return [];
    const out: { rule_key: string; incident_id: string; message: string }[] = [];
    for (const f of readiness.findings.filter((x) => x.severity === "critical")) {
      for (const incId of f.incident_ids) {
        const has = overrides.some((o) => o.rule_key === f.rule_key && o.incident_id === incId);
        if (!has) out.push({ rule_key: f.rule_key, incident_id: incId, message: f.message });
      }
    }
    return out;
  }, [readiness, overrides]);

  const openOverrideDialog = useCallback((ruleKey: string, incidentId: string) => {
    setOverrideRuleKey(ruleKey);
    setOverrideIncidentId(incidentId);
    setOverrideReason("");
    setOverrideDialogOpen(true);
  }, []);

  const submitOverride = useCallback(async () => {
    if (!activeQuarterId) return;
    try {
      await upsertOverride.mutateAsync({
        quarter_id: activeQuarterId,
        rule_key: overrideRuleKey,
        incident_id: overrideIncidentId,
        reason: overrideReason,
        approved_by: "self",
      });
      setOverrideDialogOpen(false);
      toast({ title: "Override saved" });
    } catch (err) {
      toast({ title: "Failed to save override", description: String(err), variant: "destructive" });
    }
  }, [activeQuarterId, upsertOverride, overrideRuleKey, overrideIncidentId, overrideReason]);

  const generateLeadershipPacket = useCallback(async () => {
    if (!activeQuarterId || !selectedQuarter) return;
    setPacketBusy(true);
    try {
      const dir = await open({ directory: true, multiple: false });
      if (!dir) return;

      const base = `${selectedQuarter.label} Leadership Packet`.replace(/[^a-zA-Z0-9 _-]/g, "_");
      const title = `${selectedQuarter.label} Incident Review`;
      const sections = {
        executive_summary: true,
        metrics_overview: true,
        incident_timeline: true,
        incident_breakdowns: true,
        service_reliability: true,
        qoq_comparison: true,
        discussion_points: true,
        action_items: true,
      };

      for (const format of ["docx", "pdf"] as const) {
        const tempPath = await tauriInvoke<string>("generate_report", {
          config: {
            quarter_id: activeQuarterId,
            fiscal_year: selectedQuarter.fiscal_year,
            title,
            introduction: "",
            sections,
            chart_images: {},
            format,
          },
        });
        const savePath = `${dir}/${base}.${format}`;
        await tauriInvoke("save_report", {
          tempPath,
          savePath,
          title,
          quarterId: activeQuarterId,
        });
      }

      toast({ title: "Leadership packet generated", description: `Saved DOCX + PDF to ${dir}` });
      await openPath(dir);
    } catch (err) {
      toast({ title: "Packet generation failed", description: String(err), variant: "destructive" });
    } finally {
      setPacketBusy(false);
    }
  }, [activeQuarterId, selectedQuarter]);

  return (
    <div className="p-6 max-w-5xl mx-auto space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold flex items-center gap-2">
            <FileText className="h-6 w-6" />
            Quarterly Review
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Confidence gates, metrics, and export flow for leadership-ready quarterly reporting.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {status === "ready" ? (
            <Badge className="gap-1" variant="secondary">
              <CheckCircle2 className="h-3.5 w-3.5" />
              Ready
            </Badge>
          ) : status === "warning" ? (
            <Badge className="gap-1" variant="secondary">
              <CircleAlert className="h-3.5 w-3.5" />
              Needs Review
            </Badge>
          ) : status === "needs_attention" ? (
            <Badge className="gap-1" variant="destructive">
              <AlertTriangle className="h-3.5 w-3.5" />
              Action Required
            </Badge>
          ) : (
            <Badge variant="secondary">Loading</Badge>
          )}

          {finalStatus?.finalized ? (
            <Badge className="gap-1" variant="secondary">
              <Lock className="h-3.5 w-3.5" />
              Finalized
            </Badge>
          ) : null}

          <Button
            variant="default"
            disabled={!activeQuarterId}
            onClick={() => {
              if (!activeQuarterId) return;
              navigate(`/reports?quarterId=${encodeURIComponent(activeQuarterId)}`);
            }}
          >
            Generate Packet
          </Button>
          <Button
            variant="outline"
            disabled={!activeQuarterId || packetBusy}
            onClick={generateLeadershipPacket}
          >
            {packetBusy ? "Generating…" : "Generate DOCX + PDF"}
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Quarter</CardTitle>
        </CardHeader>
        <CardContent className="grid grid-cols-3 gap-4">
          <div className="space-y-2">
            <Label>Quarter</Label>
            {quartersLoading ? (
              <div className="h-10 bg-muted animate-pulse rounded" />
            ) : (
              <Select
                value={activeQuarterId ?? ""}
                onChange={(e) => setSelectedQuarterId(e.target.value || null)}
              >
                {quarters?.map((q) => (
                  <option key={q.id} value={q.id}>
                    {q.label} (FY{q.fiscal_year})
                  </option>
                ))}
              </Select>
            )}
          </div>
          <div className="space-y-2">
            <Label>Date Range</Label>
            <div className="h-10 flex items-center rounded-md border px-3 text-sm text-muted-foreground">
              {selectedQuarter
                ? `${selectedQuarter.start_date.split("T")[0]} to ${selectedQuarter.end_date.split("T")[0]}`
                : "Select a quarter"}
            </div>
          </div>
          <div className="space-y-2">
            <Label>Inclusion Rule</Label>
            <div className="h-10 flex items-center rounded-md border px-3 text-sm text-muted-foreground">
              Included by detected_at
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Total Incidents</CardTitle>
          </CardHeader>
          <CardContent className="text-2xl font-semibold">
            {dashboardLoading ? "…" : dashboard?.total_incidents ?? 0}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">MTTR</CardTitle>
          </CardHeader>
          <CardContent className="text-2xl font-semibold">
            {dashboardLoading ? "…" : dashboard?.mttr.formatted_value ?? "—"}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">MTTA</CardTitle>
          </CardHeader>
          <CardContent className="text-2xl font-semibold">
            {dashboardLoading ? "…" : dashboard?.mtta.formatted_value ?? "—"}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Readiness</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {finalLoading ? (
            <div className="text-sm text-muted-foreground">Evaluating readiness…</div>
          ) : readiness ? (
            <>
              <div className="flex items-center gap-4 text-sm">
                <span>
                  Ready: <span className="font-medium">{readiness.ready_incidents}</span>
                </span>
                <span>
                  Needs attention:{" "}
                  <span className="font-medium">{readiness.needs_attention_incidents}</span>
                </span>
                <span>
                  Total: <span className="font-medium">{readiness.total_incidents}</span>
                </span>
              </div>

              {readiness.findings.length === 0 ? (
                <div className="text-sm text-muted-foreground">
                  No readiness findings for this quarter.
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Severity</TableHead>
                      <TableHead>Finding</TableHead>
                      <TableHead>Impacted Incidents</TableHead>
                      <TableHead>Remediation</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {readiness.findings.map((f) => (
                      <TableRow key={f.rule_key}>
                        <TableCell>{severityBadge(f.severity)}</TableCell>
                        <TableCell className="max-w-[320px]">
                          <div className="font-medium">{f.message}</div>
                        </TableCell>
                        <TableCell>
                          <div className="space-y-1">
                            <div className="text-sm text-muted-foreground">
                              {f.incident_ids.length} incident(s)
                            </div>
                            <div className="flex flex-wrap gap-2">
                              {f.incident_ids.slice(0, 6).map((id) => (
                                <Button
                                  key={id}
                                  size="sm"
                                  variant="outline"
                                  onClick={() => navigate(`/incidents/${id}`)}
                                  className="h-7 px-2"
                                >
                                  {id}
                                  <ExternalLink className="h-3.5 w-3.5 ml-2" />
                                </Button>
                              ))}
                              {f.incident_ids.length > 6 ? (
                                <span className="text-xs text-muted-foreground self-center">
                                  +{f.incident_ids.length - 6} more
                                </span>
                              ) : null}
                            </div>
                          </div>
                        </TableCell>
                        <TableCell className="max-w-[340px] text-sm text-muted-foreground">
                          {f.remediation}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </>
          ) : (
            <div className="text-sm text-muted-foreground">
              Select a quarter to compute readiness.
            </div>
          )}
        </CardContent>
      </Card>

      {activeQuarterId && readiness ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Finalization</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {finalStatus?.finalized ? (
              <div className="space-y-2 text-sm">
                <div className="flex items-center justify-between">
                  <div className="text-muted-foreground">
                    Finalized at <span className="font-medium text-foreground">{finalStatus.finalization?.finalized_at}</span>
                  </div>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => unfinalize.mutateAsync(activeQuarterId)}
                    disabled={unfinalize.isPending}
                  >
                    <Unlock className="h-4 w-4 mr-2" />
                    Unfinalize
                  </Button>
                </div>
                {finalStatus.facts_changed_since_finalization ? (
                  <div className="text-sm text-yellow-700 dark:text-yellow-400">
                    Facts changed since finalization. Re-finalize to freeze a new snapshot before leadership review.
                  </div>
                ) : (
                  <div className="text-sm text-muted-foreground">
                    Snapshot is consistent with current facts (inputs hash matches).
                  </div>
                )}
              </div>
            ) : (
              <div className="space-y-3">
                {missingOverrides.length > 0 ? (
                  <div className="text-sm text-muted-foreground">
                    {missingOverrides.length} critical item(s) need an override reason before finalizing.
                  </div>
                ) : (
                  <div className="text-sm text-muted-foreground">
                    Ready to finalize. Finalization freezes a snapshot so the leadership packet is repeatable.
                  </div>
                )}

                {missingOverrides.length > 0 ? (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Rule</TableHead>
                        <TableHead>Incident</TableHead>
                        <TableHead>Action</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {missingOverrides.slice(0, 12).map((m) => (
                        <TableRow key={`${m.rule_key}:${m.incident_id}`}>
                          <TableCell className="text-sm">{m.rule_key}</TableCell>
                          <TableCell>
                            <Button
                              size="sm"
                              variant="outline"
                              className="h-7 px-2"
                              onClick={() => navigate(`/incidents/${m.incident_id}`)}
                            >
                              {m.incident_id}
                              <ExternalLink className="h-3.5 w-3.5 ml-2" />
                            </Button>
                          </TableCell>
                          <TableCell>
                            <Button size="sm" variant="secondary" onClick={() => openOverrideDialog(m.rule_key, m.incident_id)}>
                              Add Override
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                ) : null}

                <div className="flex items-center justify-end">
                  <Button
                    onClick={() => finalize.mutateAsync({ quarter_id: activeQuarterId, finalized_by: "self", notes: "" })}
                    disabled={missingOverrides.length > 0 || finalize.isPending}
                  >
                    <Lock className="h-4 w-4 mr-2" />
                    Finalize Quarter
                  </Button>
                </div>
              </div>
            )}

            {overrides.length > 0 ? (
              <div className="space-y-2">
                <div className="text-sm font-medium">Overrides</div>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Rule</TableHead>
                      <TableHead>Incident</TableHead>
                      <TableHead>Reason</TableHead>
                      <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {overrides.map((o) => (
                      <TableRow key={o.id}>
                        <TableCell className="text-sm">{o.rule_key}</TableCell>
                        <TableCell className="text-sm">{o.incident_id}</TableCell>
                        <TableCell className="text-sm text-muted-foreground">{o.reason}</TableCell>
                        <TableCell className="text-right">
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => deleteOverride.mutateAsync({ id: o.id, quarterId: activeQuarterId })}
                            disabled={deleteOverride.isPending}
                          >
                            <Trash2 className="h-4 w-4 text-destructive" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            ) : null}
          </CardContent>
        </Card>
      ) : null}

      {glossary && glossary.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Metric Glossary</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            {glossary.map((m) => (
              <div key={m.key} className="space-y-1">
                <div className="font-medium">{m.name}</div>
                <div className="text-muted-foreground">{m.definition}</div>
                <div className="text-muted-foreground">
                  <span className="font-medium">Calc:</span> {m.calculation}
                </div>
                <div className="text-muted-foreground">
                  <span className="font-medium">Inclusion:</span> {m.inclusion}
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      ) : null}

      <Dialog open={overrideDialogOpen} onOpenChange={setOverrideDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Readiness Override</DialogTitle>
            <DialogDescription>
              Record why you are accepting a critical gap for leadership reporting.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-2 text-sm">
            <div className="text-muted-foreground">
              Rule: <span className="font-medium text-foreground">{overrideRuleKey}</span>
            </div>
            <div className="text-muted-foreground">
              Incident: <span className="font-medium text-foreground">{overrideIncidentId}</span>
            </div>
          </div>

          <div className="space-y-2">
            <Label>Reason</Label>
            <Textarea
              value={overrideReason}
              onChange={(e) => setOverrideReason(e.target.value)}
              placeholder="Explain why this is acceptable for the quarter packet (what is missing, why, and any mitigation)."
            />
          </div>

          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setOverrideDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={submitOverride} disabled={!overrideReason.trim() || upsertOverride.isPending}>
              Save Override
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
