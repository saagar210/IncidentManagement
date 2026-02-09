import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { FieldProvenance } from "@/types/provenance";

export function useFieldProvenance(entityType: string, entityId: string | null) {
  return useQuery({
    queryKey: ["provenance", entityType, entityId],
    queryFn: () =>
      tauriInvoke<FieldProvenance[]>("list_field_provenance_for_entity", {
        entityType,
        entityId,
      }),
    enabled: !!entityId,
  });
}

