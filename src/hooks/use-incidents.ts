import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  Incident,
  CreateIncidentRequest,
  UpdateIncidentRequest,
  IncidentFilters,
  ActionItem,
  CreateActionItemRequest,
  UpdateActionItemRequest,
} from "@/types/incident";

export function useIncidents(filters: IncidentFilters = {}) {
  return useQuery({
    queryKey: ["incidents", filters],
    queryFn: () => tauriInvoke<Incident[]>("list_incidents", { filters }),
  });
}

export function useIncident(id: string | undefined) {
  const isValidId = !!id && id !== "new";
  return useQuery({
    queryKey: ["incident", id],
    queryFn: () => tauriInvoke<Incident>("get_incident", { id }),
    enabled: isValidId,
  });
}

export function useSearchIncidents(query: string) {
  return useQuery({
    queryKey: ["incidents-search", query],
    queryFn: () => tauriInvoke<Incident[]>("search_incidents", { query }),
    enabled: query.length > 0,
  });
}

export function useCreateIncident() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (incident: CreateIncidentRequest) =>
      tauriInvoke<Incident>("create_incident", { incident }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incidents"] });
    },
  });
}

export function useUpdateIncident() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      incident,
    }: {
      id: string;
      incident: UpdateIncidentRequest;
    }) => tauriInvoke<Incident>("update_incident", { id, incident }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["incidents"] });
      queryClient.invalidateQueries({
        queryKey: ["incident", variables.id],
      });
    },
  });
}

export function useDeleteIncident() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_incident", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incidents"] });
      queryClient.invalidateQueries({ queryKey: ["deleted-count"] });
      queryClient.invalidateQueries({ queryKey: ["deleted-incidents"] });
    },
  });
}

export function useBulkUpdateStatus() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ ids, status }: { ids: string[]; status: string }) =>
      tauriInvoke<void>("bulk_update_status", { ids, status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incidents"] });
    },
  });
}

// Action Items

export function useActionItems(incidentId?: string) {
  return useQuery({
    queryKey: ["action-items", incidentId],
    queryFn: () =>
      tauriInvoke<ActionItem[]>("list_action_items", {
        incidentId: incidentId ?? null,
      }),
  });
}

export function useCreateActionItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (item: CreateActionItemRequest) =>
      tauriInvoke<ActionItem>("create_action_item", { item }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["action-items"] });
    },
  });
}

export function useUpdateActionItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, item }: { id: string; item: UpdateActionItemRequest }) =>
      tauriInvoke<ActionItem>("update_action_item", { id, item }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["action-items"] });
    },
  });
}

export function useDeleteActionItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_action_item", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["action-items"] });
    },
  });
}
