import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";

export interface StakeholderUpdate {
  id: string;
  incident_id: string;
  content: string;
  update_type: string;
  generated_by: string;
  created_at: string;
}

export interface CreateStakeholderUpdateRequest {
  incident_id: string;
  content: string;
  update_type?: string;
  generated_by?: string;
}

export function useStakeholderUpdates(incidentId: string) {
  return useQuery({
    queryKey: ["stakeholder-updates", incidentId],
    queryFn: () =>
      tauriInvoke<StakeholderUpdate[]>("list_stakeholder_updates", {
        incidentId,
      }),
    enabled: !!incidentId,
  });
}

export function useCreateStakeholderUpdate() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateStakeholderUpdateRequest) =>
      tauriInvoke<StakeholderUpdate>("create_stakeholder_update", { req }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({
        queryKey: ["stakeholder-updates", vars.incident_id],
      });
    },
  });
}

export function useDeleteStakeholderUpdate() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_stakeholder_update", { id }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["stakeholder-updates"] });
    },
  });
}
