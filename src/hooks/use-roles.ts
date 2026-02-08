import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { IncidentRole, AssignRoleRequest } from "@/types/role";

export function useIncidentRoles(incidentId: string | undefined) {
  return useQuery({
    queryKey: ["incident-roles", incidentId],
    queryFn: () =>
      tauriInvoke<IncidentRole[]>("list_incident_roles", { incidentId }),
    enabled: !!incidentId,
  });
}

export function useAssignRole() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: AssignRoleRequest) =>
      tauriInvoke<IncidentRole>("assign_role", { req }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["incident-roles", variables.incident_id],
      });
      queryClient.invalidateQueries({ queryKey: ["notification-summary"] });
    },
  });
}

export function useUnassignRole() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => tauriInvoke<void>("unassign_role", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incident-roles"] });
      queryClient.invalidateQueries({ queryKey: ["notification-summary"] });
    },
  });
}
