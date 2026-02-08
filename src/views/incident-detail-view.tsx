import { useEffect, useMemo } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useForm } from "react-hook-form";
import { ArrowLeft, Trash2, Save } from "lucide-react";
import {
  useIncident,
  useCreateIncident,
  useUpdateIncident,
  useDeleteIncident,
} from "@/hooks/use-incidents";
import { useActiveServices } from "@/hooks/use-services";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
  const severityMap: Record<string, number> = { Critical: 0, High: 1, Medium: 2, Low: 3 };
  const impactMap: Record<string, number> = { Critical: 0, High: 1, Medium: 2, Low: 3 };
  const s = severityMap[severity] ?? 3;
  const i = impactMap[impact] ?? 3;
  const avg = (s + i) / 2;
  if (avg <= 0.5) return "P0";
  if (avg <= 1.5) return "P1";
  if (avg <= 2) return "P2";
  if (avg <= 2.5) return "P3";
  return "P4";
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
  const isEditMode = !!id;

  const { data: incident, isLoading: incidentLoading } = useIncident(id);
  const { data: services } = useActiveServices();
  const createMutation = useCreateIncident();
  const updateMutation = useUpdateIncident();
  const deleteMutation = useDeleteIncident();

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

  const onSubmit = async (data: IncidentFormData) => {
    const payload: CreateIncidentRequest = {
      title: data.title,
      service_id: data.service_id,
      severity: data.severity,
      impact: data.impact,
      status: data.status,
      started_at: toISOString(data.started_at) ?? new Date().toISOString(),
      detected_at: toISOString(data.detected_at) ?? new Date().toISOString(),
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

    if (isEditMode && id) {
      await updateMutation.mutateAsync({ id, incident: payload });
    } else {
      await createMutation.mutateAsync(payload);
    }
    navigate("/incidents");
  };

  const handleDelete = async () => {
    if (!id) return;
    const confirmed = window.confirm(
      "Are you sure you want to delete this incident? This cannot be undone."
    );
    if (!confirmed) return;
    await deleteMutation.mutateAsync(id);
    navigate("/incidents");
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

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Section 1: Identification */}
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
          </CardContent>
        </Card>

        {/* Section 2: Classification */}
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

        {/* Section 3: Timeline */}
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

        {/* Section 4: Impact Assessment */}
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

        {/* Section 5: Recurrence */}
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

        {/* Section 6: Analysis */}
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Analysis</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label htmlFor="root_cause">Root Cause</Label>
              <Textarea
                id="root_cause"
                rows={3}
                placeholder="What caused this incident?"
                {...register("root_cause")}
              />
            </div>
            <div>
              <Label htmlFor="resolution">Resolution</Label>
              <Textarea
                id="resolution"
                rows={3}
                placeholder="How was this resolved?"
                {...register("resolution")}
              />
            </div>
            <div>
              <Label htmlFor="lessons_learned">Lessons Learned</Label>
              <Textarea
                id="lessons_learned"
                rows={3}
                placeholder="What did we learn?"
                {...register("lessons_learned")}
              />
            </div>
            <div>
              <Label htmlFor="notes">Notes</Label>
              <Textarea
                id="notes"
                rows={3}
                placeholder="Additional notes"
                {...register("notes")}
              />
            </div>
          </CardContent>
        </Card>
      </form>
    </div>
  );
}
