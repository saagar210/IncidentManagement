import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  ChecklistTemplate,
  CreateChecklistTemplateRequest,
  UpdateChecklistTemplateRequest,
  IncidentChecklist,
  CreateIncidentChecklistRequest,
  ChecklistItem,
  ToggleChecklistItemRequest,
} from "@/types/checklist";

// ── Template Hooks ────────────────────────────────────────────────

export function useChecklistTemplates() {
  return useQuery({
    queryKey: ["checklist-templates"],
    queryFn: () =>
      tauriInvoke<ChecklistTemplate[]>("list_checklist_templates"),
  });
}

export function useCreateChecklistTemplate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateChecklistTemplateRequest) =>
      tauriInvoke<ChecklistTemplate>("create_checklist_template", { req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["checklist-templates"] });
    },
  });
}

export function useUpdateChecklistTemplate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      req,
    }: {
      id: string;
      req: UpdateChecklistTemplateRequest;
    }) => tauriInvoke<ChecklistTemplate>("update_checklist_template", { id, req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["checklist-templates"] });
    },
  });
}

export function useDeleteChecklistTemplate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_checklist_template", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["checklist-templates"] });
    },
  });
}

// ── Incident Checklist Hooks ──────────────────────────────────────

export function useIncidentChecklists(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["incident-checklists", incidentId],
    queryFn: () =>
      tauriInvoke<IncidentChecklist[]>("list_incident_checklists", {
        incidentId,
      }),
    enabled: !!incidentId,
  });
}

export function useCreateIncidentChecklist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateIncidentChecklistRequest) =>
      tauriInvoke<IncidentChecklist>("create_incident_checklist", { req }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["incident-checklists", variables.incident_id],
      });
      queryClient.invalidateQueries({ queryKey: ["notification-summary"] });
    },
  });
}

export function useDeleteIncidentChecklist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_incident_checklist", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incident-checklists"] });
    },
  });
}

export function useToggleChecklistItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      itemId,
      req,
    }: {
      itemId: string;
      req: ToggleChecklistItemRequest;
    }) => tauriInvoke<ChecklistItem>("toggle_checklist_item", { itemId, req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incident-checklists"] });
    },
  });
}
