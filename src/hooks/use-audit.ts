import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { AuditEntry, AuditFilters, NotificationSummary } from "@/types/audit";

export function useAuditLog(filters: AuditFilters = {}) {
  return useQuery({
    queryKey: ["audit-entries", filters],
    queryFn: () => tauriInvoke<AuditEntry[]>("list_audit_entries", { filters }),
  });
}

export function useEntityAuditLog(entityType: string, entityId: string | undefined) {
  return useQuery({
    queryKey: ["audit-entries", entityType, entityId],
    queryFn: () =>
      tauriInvoke<AuditEntry[]>("list_audit_entries", {
        filters: { entity_type: entityType, entity_id: entityId, limit: 50 },
      }),
    enabled: !!entityId,
  });
}

export function useNotificationSummary() {
  return useQuery({
    queryKey: ["notification-summary"],
    queryFn: () => tauriInvoke<NotificationSummary>("get_notification_summary"),
    staleTime: 30_000,
    refetchInterval: 60_000,
  });
}
