import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { ServiceAlias } from "@/types/service-aliases";

export function useServiceAliases() {
  return useQuery({
    queryKey: ["service-aliases"],
    queryFn: () => tauriInvoke<ServiceAlias[]>("list_service_aliases"),
  });
}

export function useCreateServiceAlias() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: { alias: string; service_id: string }) =>
      tauriInvoke<ServiceAlias>("create_service_alias", { req }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["service-aliases"] });
    },
  });
}

export function useDeleteServiceAlias() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => tauriInvoke<void>("delete_service_alias", { id }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["service-aliases"] });
    },
  });
}

