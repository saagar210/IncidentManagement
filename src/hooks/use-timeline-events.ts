import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { TimelineEvent, CreateTimelineEventRequest, TimelineImportResult } from "@/types/timeline-events";

export function useTimelineEvents(incidentId: string | null) {
  return useQuery({
    queryKey: ["timeline-events", incidentId],
    queryFn: () =>
      tauriInvoke<TimelineEvent[]>("list_timeline_events_for_incident", { incidentId }),
    enabled: !!incidentId,
  });
}

export function useCreateTimelineEvent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateTimelineEventRequest) =>
      tauriInvoke<TimelineEvent>("create_timeline_event", { req }),
    onSuccess: (_data, req) => {
      qc.invalidateQueries({ queryKey: ["timeline-events", req.incident_id] });
      qc.invalidateQueries({ queryKey: ["audit", "incident", req.incident_id] });
    },
  });
}

export function useDeleteTimelineEvent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (vars: { id: string; incidentId: string }) =>
      tauriInvoke<void>("delete_timeline_event", { id: vars.id }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["timeline-events", vars.incidentId] });
    },
  });
}

export function useImportTimelineFromPaste() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (vars: { incident_id: string; paste_text: string; source?: string }) =>
      tauriInvoke<TimelineImportResult>("import_timeline_events_from_paste", {
        incidentId: vars.incident_id,
        pasteText: vars.paste_text,
        source: vars.source,
      }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["timeline-events", vars.incident_id] });
    },
  });
}

export function useImportTimelineFromJson() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (vars: { incident_id: string; json_str: string; source?: string }) =>
      tauriInvoke<TimelineImportResult>("import_timeline_events_from_json", {
        incidentId: vars.incident_id,
        jsonStr: vars.json_str,
        source: vars.source,
      }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["timeline-events", vars.incident_id] });
    },
  });
}

