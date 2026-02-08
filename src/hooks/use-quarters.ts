import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { QuarterConfig, UpsertQuarterRequest } from "@/types/incident";

export function useQuarters() {
  return useQuery({
    queryKey: ["quarters"],
    queryFn: () => tauriInvoke<QuarterConfig[]>("get_quarter_configs"),
  });
}

export function useUpsertQuarter() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (config: UpsertQuarterRequest) =>
      tauriInvoke<QuarterConfig>("upsert_quarter_config", { config }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["quarters"] });
    },
  });
}

export function useDeleteQuarter() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_quarter_config", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["quarters"] });
    },
  });
}
