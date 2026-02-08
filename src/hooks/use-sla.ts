import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  SlaDefinition,
  CreateSlaDefinitionRequest,
  UpdateSlaDefinitionRequest,
  SlaStatus,
} from "@/types/sla";

export function useSlaDefinitions() {
  return useQuery({
    queryKey: ["sla-definitions"],
    queryFn: () => tauriInvoke<SlaDefinition[]>("list_sla_definitions"),
  });
}

export function useSlaStatus(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["sla-status", incidentId],
    queryFn: () =>
      tauriInvoke<SlaStatus>("compute_sla_status", {
        incidentId,
      }),
    enabled: !!incidentId,
    refetchInterval: 60_000, // Re-check SLA status every minute for active incidents
  });
}

export function useCreateSlaDefinition() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateSlaDefinitionRequest) =>
      tauriInvoke<SlaDefinition>("create_sla_definition", { req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sla-definitions"] });
      queryClient.invalidateQueries({ queryKey: ["sla-status"] });
    },
  });
}

export function useUpdateSlaDefinition() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, req }: { id: string; req: UpdateSlaDefinitionRequest }) =>
      tauriInvoke<SlaDefinition>("update_sla_definition", { id, req }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sla-definitions"] });
      queryClient.invalidateQueries({ queryKey: ["sla-status"] });
    },
  });
}

export function useDeleteSlaDefinition() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_sla_definition", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sla-definitions"] });
      queryClient.invalidateQueries({ queryKey: ["sla-status"] });
    },
  });
}
