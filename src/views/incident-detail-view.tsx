import { useEffect, useMemo, useState, useRef, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useForm } from "react-hook-form";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Trash2, Save } from "lucide-react";
import {
  useIncident,
  useCreateIncident,
  useUpdateIncident,
  useDeleteIncident,
} from "@/hooks/use-incidents";
import { useActiveServices } from "@/hooks/use-services";
import { tauriInvoke } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
import { TagInput } from "@/components/ui/tag-input";
import { AttachmentList } from "@/components/incidents/attachment-list";
import { CustomFieldsForm } from "@/components/incidents/custom-fields-form";
import { ActionItemsPanel } from "@/components/incidents/ActionItemsPanel";
import { SlaStatusBadge } from "@/components/incidents/SlaStatusBadge";
import { ActivityFeed } from "@/components/incidents/ActivityFeed";
import { RoleAssignmentPanel } from "@/components/incidents/role-assignment-panel";
import { ChecklistPanel } from "@/components/incidents/checklist-panel";
import { StatusTransitionBar } from "@/components/incidents/status-transition-bar";
import { IncidentTimelineStream } from "@/components/incidents/incident-timeline-stream";
import { ContributingFactorsForm } from "@/components/incidents/contributing-factors-form";
import { PostmortemEditor } from "@/components/incidents/postmortem-editor";
import { AiSummaryPanel } from "@/components/ai/ai-summary-panel";
import { SimilarIncidentsPanel } from "@/components/ai/similar-incidents-panel";
import { RootCauseSuggestions } from "@/components/ai/root-cause-suggestions";
import { DedupWarning } from "@/components/ai/dedup-warning";
import { StakeholderUpdatePanel } from "@/components/incidents/stakeholder-update-panel";
import { toast } from "@/components/ui/use-toast";
import {
  SEVERITY_LEVELS,
  IMPACT_LEVELS,
  STATUS_OPTIONS,
} from "@/lib/constants";
import type { CreateIncidentRequest } from "@/types/incident";

interface IncidentFormData {
  title: string;
  service_id: string;
  external_ref: string;
  severity: string;
  impact: string;
  status: string;
  started_at: string;
  detected_at: string;
  acknowledged_at: string;
  first_response_at: string;
  mitigation_started_at: string;
  responded_at: string;
  resolved_at: string;
  tickets_submitted: number;
  affected_users: number;
  is_recurring: boolean;
  recurrence_of: string;
  root_cause: string;
  resolution: string;
  lessons_learned: string;
  notes: string;
}

function computePriority(severity: string, impact: string): string {
  // Must match the Rust priority matrix exactly
  const matrix: Record<string, Record<string, string>> = {
    Critical: { Critical: "P0", High: "P1", Medium: "P1", Low: "P2" },
    High:     { Critical: "P1", High: "P1", Medium: "P2", Low: "P3" },
    Medium:   { Critical: "P2", High: "P2", Medium: "P3", Low: "P3" },
    Low:      { Critical: "P3", High: "P3", Medium: "P4", Low: "P4" },
  };
  return matrix[severity]?.[impact] ?? "P4";
}

function toLocalDatetime(isoStr: string | null | undefined): string {
  if (!isoStr) return "";
  try {
    const d = new Date(isoStr);
    // Format as YYYY-MM-DDTHH:MM for datetime-local input
    const pad = (n: number) => n.toString().padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  } catch {
    return "";
  }
}

function toISOString(localDatetime: string): string | null {
  if (!localDatetime) return null;
  try {
    return new Date(localDatetime).toISOString();
  } catch {
    return null;
  }
}

function computeDuration(started: string, resolved: string): string {
  if (!started || !resolved) return "--";
  try {
    const start = new Date(started).getTime();
    const end = new Date(resolved).getTime();
    const mins = Math.round((end - start) / 60000);
    if (mins < 0) return "--";
    if (mins < 60) return `${mins} minutes`;
    const hours = Math.floor(mins / 60);
    const remainMins = mins % 60;
    if (hours < 24) return `${hours}h ${remainMins}m`;
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h ${remainMins}m`;
  } catch {
    return "--";
  }
}

export function IncidentDetailView() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const isEditMode = !!id && id !== "new";

  const { data: incident, isLoading: incidentLoading } = useIncident(isEditMode ? id : undefined);
  const { data: services } = useActiveServices();
  const createMutation = useCreateIncident();
  const updateMutation = useUpdateIncident();
  const deleteMutation = useDeleteIncident();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<
    "details" | "analysis" | "actions" | "postmortem" | "activity"
  >("details");

  // Tags
  const [tags, setTags] = useState<string[]>([]);
  const { data: existingTags } = useQuery({
    queryKey: ["incident-tags", id],
    queryFn: () => tauriInvoke<string[]>("get_incident_tags", { incidentId: id as string }),
    enabled: isEditMode,
  });
  const { data: allTags } = useQuery({
    queryKey: ["all-tags"],
    queryFn: () => tauriInvoke<string[]>("get_all_tags"),
  });
  useEffect(() => {
    if (existingTags) setTags(existingTags);
  }, [existingTags]);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { isDirty, isSubmitting },
  } = useForm<IncidentFormData>({
    defaultValues: {
      title: "",
      service_id: "",
      external_ref: "",
      severity: "Medium",
      impact: "Medium",
      status: "Active",
      started_at: "",
      detected_at: "",
      acknowledged_at: "",
      first_response_at: "",
      mitigation_started_at: "",
      responded_at: "",
      resolved_at: "",
      tickets_submitted: 0,
      affected_users: 0,
      is_recurring: false,
      recurrence_of: "",
      root_cause: "",
      resolution: "",
      lessons_learned: "",
      notes: "",
    },
  });

  // Populate form when incident data loads (edit mode)
  useEffect(() => {
    if (incident && isEditMode) {
      reset({
        title: incident.title,
        service_id: incident.service_id,
        external_ref: incident.external_ref ?? "",
        severity: incident.severity,
        impact: incident.impact,
        status: incident.status,
        started_at: toLocalDatetime(incident.started_at),
        detected_at: toLocalDatetime(incident.detected_at),
        acknowledged_at: toLocalDatetime(incident.acknowledged_at),
        first_response_at: toLocalDatetime(incident.first_response_at),
        mitigation_started_at: toLocalDatetime(incident.mitigation_started_at),
        responded_at: toLocalDatetime(incident.responded_at),
        resolved_at: toLocalDatetime(incident.resolved_at),
        tickets_submitted: incident.tickets_submitted ?? 0,
        affected_users: incident.affected_users ?? 0,
        is_recurring: incident.is_recurring ?? false,
        recurrence_of: incident.recurrence_of ?? "",
        root_cause: incident.root_cause ?? "",
        resolution: incident.resolution ?? "",
        lessons_learned: incident.lessons_learned ?? "",
        notes: incident.notes ?? "",
      });
    }
  }, [incident, isEditMode, reset]);

  const watchedSeverity = watch("severity");
  const watchedImpact = watch("impact");
  const watchedServiceId = watch("service_id");
  const watchedStartedAt = watch("started_at");
  const watchedResolvedAt = watch("resolved_at");

  const computedPriority = useMemo(
    () => computePriority(watchedSeverity, watchedImpact),
    [watchedSeverity, watchedImpact]
  );

  const computedDuration = useMemo(
    () => computeDuration(watchedStartedAt, watchedResolvedAt),
    [watchedStartedAt, watchedResolvedAt]
  );

  // Auto-fill severity/impact when service changes (only in create mode)
  useEffect(() => {
    if (!isEditMode && watchedServiceId && services) {
      const service = services.find((s) => s.id === watchedServiceId);
      if (service) {
        setValue("severity", service.default_severity);
        setValue("impact", service.default_impact);
      }
    }
  }, [watchedServiceId, services, isEditMode, setValue]);

  // Callback for custom fields save
  const customFieldsSaveRef = useRef<(() => Promise<void>) | null>(null);
  const handleCustomFieldsSaveRef = useCallback(
    (saveFn: (() => Promise<void>) | null) => {
      customFieldsSaveRef.current = saveFn;
    },
    []
  );

  const onSubmit = async (data: IncidentFormData) => {
    const payload: CreateIncidentRequest = {
      title: data.title,
      service_id: data.service_id,
      severity: data.severity,
      impact: data.impact,
      status: data.status,
      started_at: toISOString(data.started_at) ?? new Date().toISOString(),
      detected_at: toISOString(data.detected_at) ?? new Date().toISOString(),
      acknowledged_at: toISOString(data.acknowledged_at),
      first_response_at: toISOString(data.first_response_at),
      mitigation_started_at: toISOString(data.mitigation_started_at),
      responded_at: toISOString(data.responded_at),
      resolved_at: toISOString(data.resolved_at),
      tickets_submitted: data.tickets_submitted,
      affected_users: data.affected_users,
      is_recurring: data.is_recurring,
      recurrence_of: data.recurrence_of || null,
      root_cause: data.root_cause,
      resolution: data.resolution,
      lessons_learned: data.lessons_learned,
      external_ref: data.external_ref,
      notes: data.notes,
    };

    try {
      let incidentId: string;
      if (isEditMode && id) {
        await updateMutation.mutateAsync({ id, incident: payload });
        incidentId = id;
      } else {
        const created = await createMutation.mutateAsync(payload);
        incidentId = created.id;
      }

      // Save tags
      await tauriInvoke("set_incident_tags", {
        incidentId,
        tags,
      });
      queryClient.invalidateQueries({ queryKey: ["incident-tags"] });
      queryClient.invalidateQueries({ queryKey: ["all-tags"] });

      // Save custom fields
      if (customFieldsSaveRef.current) {
        await customFieldsSaveRef.current();
      }

      navigate("/incidents");
    } catch (err) {
      toast({
        title: "Save failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const handleDelete = async () => {
    if (!id) return;
    const confirmed = window.confirm(
      "Move this incident to trash?"
    );
    if (!confirmed) return;
    try {
      await deleteMutation.mutateAsync(id);
      navigate("/incidents");
    } catch {
      // Error is handled by global mutation error handler
    }
  };

  if (isEditMode && incidentLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-muted-foreground">Loading incident...</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={() => navigate("/incidents")}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <h1 className="text-2xl font-semibold">
            {isEditMode ? "Edit Incident" : "New Incident"}
          </h1>
        </div>
        <div className="flex items-center gap-2">
          {isEditMode && (
            <Button
              variant="destructive"
              size="sm"
              onClick={handleDelete}
              disabled={deleteMutation.isPending}
            >
              <Trash2 className="h-4 w-4" />
              Delete
            </Button>
          )}
          <Button variant="outline" onClick={() => navigate("/incidents")}>
            Cancel
          </Button>
          <Button
            onClick={handleSubmit(onSubmit)}
            disabled={isSubmitting || (!isDirty && isEditMode)}
          >
            <Save className="h-4 w-4" />
            {isEditMode ? "Save Changes" : "Create Incident"}
          </Button>
        </div>
      </div>

      {/* Tabbed Form */}
      <form onSubmit={handleSubmit(onSubmit)}>
        <Tabs
          value={activeTab}
          onValueChange={(v) =>
            setActiveTab(v as "details" | "analysis" | "actions" | "postmortem" | "activity")
          }
        >
          <TabsList className="w-full justify-start">
            <TabsTrigger value="details">Details</TabsTrigger>
            <TabsTrigger value="analysis">Analysis</TabsTrigger>
            {isEditMode && <TabsTrigger value="actions">Actions & Extras</TabsTrigger>}
            {isEditMode && <TabsTrigger value="postmortem">Post-Mortem</TabsTrigger>}
            {isEditMode && <TabsTrigger value="activity">Activity</TabsTrigger>}
          </TabsList>

          {/* Tab 1: Details — Identification, Classification, SLA, Timeline, Impact, Recurrence */}
          <TabsContent value="details" className="space-y-6">
            {/* Identification */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Identification</CardTitle>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div className="sm:col-span-2">
                  <Label htmlFor="title">Title *</Label>
                  <Input
                    id="title"
                    placeholder="Brief description of the incident"
                    {...register("title", { required: true })}
                  />
                </div>
                <div>
                  <Label htmlFor="service_id">Service *</Label>
                  <Select
                    id="service_id"
                    {...register("service_id", { required: true })}
                  >
                    <option value="">Select a service</option>
                    {services?.map((s) => (
                      <option key={s.id} value={s.id}>
                        {s.name}
                      </option>
                    ))}
                  </Select>
                </div>
                <div>
                  <Label htmlFor="external_ref">External Reference</Label>
                  <Input
                    id="external_ref"
                    placeholder="e.g., JIRA-1234"
                    {...register("external_ref")}
                  />
                </div>
                <div className="sm:col-span-2">
                  <Label>Tags</Label>
                  <TagInput
                    tags={tags}
                    onChange={setTags}
                    suggestions={allTags ?? []}
                    placeholder="Add tags..."
                  />
                </div>
              </CardContent>
            </Card>

            {/* Dedup Warning (create mode only) */}
            {!isEditMode && (
              <DedupWarning
                title={watch("title")}
                serviceId={watch("service_id")}
              />
            )}

            {/* Classification */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Classification</CardTitle>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <div>
                  <Label htmlFor="severity">Severity</Label>
                  <Select id="severity" {...register("severity")}>
                    {SEVERITY_LEVELS.map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </Select>
                </div>
                <div>
                  <Label htmlFor="impact">Impact</Label>
                  <Select id="impact" {...register("impact")}>
                    {IMPACT_LEVELS.map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </Select>
                </div>
                <div>
                  <Label>Priority (computed)</Label>
                  <div className="flex h-9 items-center rounded-md border bg-muted px-3 text-sm font-medium">
                    {computedPriority}
                  </div>
                </div>
                <div>
                  <Label htmlFor="status">Status</Label>
                  <Select id="status" {...register("status")}>
                    {STATUS_OPTIONS.map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </Select>
                </div>
              </CardContent>
            </Card>

            {/* Status Transition Bar (edit mode only) */}
            {isEditMode && incident && (
              <StatusTransitionBar
                currentStatus={watch("status")}
                reopenCount={incident.reopen_count ?? 0}
                disabled={isSubmitting}
                onTransition={(newStatus) => {
                  setValue("status", newStatus, { shouldDirty: true });
                }}
              />
            )}

            {/* SLA Status (existing incidents only) */}
            {id && <SlaStatusBadge incidentId={id} />}

            {/* Timeline */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Timeline</CardTitle>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <div>
                  <Label htmlFor="started_at">Started At *</Label>
                  <Input
                    id="started_at"
                    type="datetime-local"
                    {...register("started_at", { required: true })}
                  />
                </div>
                <div>
                  <Label htmlFor="detected_at">Detected At *</Label>
                  <Input
                    id="detected_at"
                    type="datetime-local"
                    {...register("detected_at", { required: true })}
                  />
                </div>
                <div>
                  <Label htmlFor="acknowledged_at">Acknowledged At</Label>
                  <Input
                    id="acknowledged_at"
                    type="datetime-local"
                    {...register("acknowledged_at")}
                  />
                </div>
                <div>
                  <Label htmlFor="first_response_at">First Response At</Label>
                  <Input
                    id="first_response_at"
                    type="datetime-local"
                    {...register("first_response_at")}
                  />
                </div>
                <div>
                  <Label htmlFor="mitigation_started_at">Mitigation Started</Label>
                  <Input
                    id="mitigation_started_at"
                    type="datetime-local"
                    {...register("mitigation_started_at")}
                  />
                </div>
                <div>
                  <Label htmlFor="responded_at">Responded At</Label>
                  <Input
                    id="responded_at"
                    type="datetime-local"
                    {...register("responded_at")}
                  />
                </div>
                <div>
                  <Label htmlFor="resolved_at">Resolved At</Label>
                  <Input
                    id="resolved_at"
                    type="datetime-local"
                    {...register("resolved_at")}
                  />
                </div>
                <div>
                  <Label>Duration (computed)</Label>
                  <div className="flex h-9 items-center rounded-md border bg-muted px-3 text-sm">
                    {computedDuration}
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Impact Assessment */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Impact Assessment</CardTitle>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div>
                  <Label htmlFor="tickets_submitted">Tickets Submitted</Label>
                  <Input
                    id="tickets_submitted"
                    type="number"
                    min={0}
                    {...register("tickets_submitted", { valueAsNumber: true })}
                  />
                </div>
                <div>
                  <Label htmlFor="affected_users">Affected Users</Label>
                  <Input
                    id="affected_users"
                    type="number"
                    min={0}
                    {...register("affected_users", { valueAsNumber: true })}
                  />
                </div>
              </CardContent>
            </Card>

            {/* Recurrence */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Recurrence</CardTitle>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div className="flex items-center gap-3">
                  <input
                    id="is_recurring"
                    type="checkbox"
                    className="h-4 w-4 rounded border-input"
                    {...register("is_recurring")}
                  />
                  <Label htmlFor="is_recurring" className="mb-0">
                    Is Recurring Incident
                  </Label>
                </div>
                <div>
                  <Label htmlFor="recurrence_of">Recurrence Of (Incident ID)</Label>
                  <Input
                    id="recurrence_of"
                    placeholder="UUID of original incident"
                    {...register("recurrence_of")}
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Tab 2: Analysis — Root Cause, Resolution, Lessons Learned, Notes */}
          <TabsContent value="analysis" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Analysis</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label>Root Cause</Label>
                  <MarkdownEditor
                    value={watch("root_cause")}
                    onChange={(val) => setValue("root_cause", val, { shouldDirty: true })}
                    placeholder="What caused this incident?"
                  />
                </div>
                <div>
                  <Label>Resolution</Label>
                  <MarkdownEditor
                    value={watch("resolution")}
                    onChange={(val) => setValue("resolution", val, { shouldDirty: true })}
                    placeholder="How was this resolved?"
                  />
                </div>
                <div>
                  <Label>Lessons Learned</Label>
                  <MarkdownEditor
                    value={watch("lessons_learned")}
                    onChange={(val) => setValue("lessons_learned", val, { shouldDirty: true })}
                    placeholder="What did we learn?"
                  />
                </div>
                <div>
                  <Label>Notes</Label>
                  <MarkdownEditor
                    value={watch("notes")}
                    onChange={(val) => setValue("notes", val, { shouldDirty: true })}
                    placeholder="Additional notes"
                    height={150}
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Tab 3: Actions & Extras — Roles, Checklists, Action Items, Custom Fields, Attachments (edit only) */}
          {isEditMode && id && (
            <TabsContent value="actions" className="space-y-6">
              <RoleAssignmentPanel incidentId={id} />
              <ChecklistPanel incidentId={id} />
              <StakeholderUpdatePanel
                incidentId={id}
                title={watch("title")}
                severity={watch("severity")}
                status={watch("status")}
                service={services?.find((s) => s.id === watch("service_id"))?.name ?? ""}
                impact={watch("impact")}
                notes={watch("notes")}
              />
              <ActionItemsPanel incidentId={id} />
              <CustomFieldsForm
                incidentId={id}
                onSaveRef={handleCustomFieldsSaveRef}
              />
              <AttachmentList incidentId={id} />
            </TabsContent>
          )}

          {/* Tab: Post-Mortem — Contributing Factors, PM Editor, AI (edit only) */}
          {isEditMode && id && (
            <TabsContent value="postmortem" className="space-y-6">
              <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
                <div className="space-y-6 lg:col-span-2">
                  <div id="pir-contributing-factors">
                    <ContributingFactorsForm incidentId={id} />
                  </div>
                  <PostmortemEditor
                    incidentId={id}
                    title={watch("title")}
                    severity={watch("severity")}
                    service={services?.find((s) => s.id === watch("service_id"))?.name ?? ""}
                    rootCause={watch("root_cause")}
                    resolution={watch("resolution")}
                    lessons={watch("lessons_learned")}
                    onNavigateToTab={(tab) => setActiveTab(tab)}
                  />
                </div>
                <div className="space-y-4">
                  <AiSummaryPanel
                    title={watch("title")}
                    severity={watch("severity")}
                    status={watch("status")}
                    service={services?.find((s) => s.id === watch("service_id"))?.name ?? ""}
                    impact={watch("impact")}
                    rootCause={watch("root_cause")}
                    resolution={watch("resolution")}
                    notes={watch("notes")}
                  />
                  <RootCauseSuggestions
                    title={watch("title")}
                    severity={watch("severity")}
                    service={services?.find((s) => s.id === watch("service_id"))?.name ?? ""}
                    symptoms={watch("root_cause")}
                    timeline={watch("notes")}
                  />
                  <SimilarIncidentsPanel
                    query={watch("title")}
                    excludeId={id}
                  />
                </div>
              </div>
            </TabsContent>
          )}

          {/* Tab: Activity — Audit Log (edit only) */}
          {isEditMode && id && (
            <TabsContent value="activity" className="space-y-6">
              <IncidentTimelineStream incidentId={id} />
              <Card>
                <CardHeader>
                  <CardTitle className="text-base">Activity Log</CardTitle>
                </CardHeader>
                <CardContent>
                  <ActivityFeed incidentId={id} />
                </CardContent>
              </Card>
            </TabsContent>
          )}
        </Tabs>
      </form>
    </div>
  );
}
