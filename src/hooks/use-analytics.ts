import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type {
  BacklogAgingBucket,
  ServiceReliabilityScore,
  EscalationFunnelEntry,
} from "@/types/analytics";

export function useBacklogAging() {
  return useQuery({
    queryKey: ["backlog-aging"],
    queryFn: () => tauriInvoke<BacklogAgingBucket[]>("get_backlog_aging"),
    staleTime: 30000,
  });
}

export function useServiceReliability(startDate: string, endDate: string) {
  return useQuery({
    queryKey: ["service-reliability", startDate, endDate],
    queryFn: () =>
      tauriInvoke<ServiceReliabilityScore[]>("get_service_reliability", {
        startDate,
        endDate,
      }),
    enabled: !!startDate && !!endDate,
    staleTime: 30000,
  });
}

export function useEscalationFunnel(startDate: string, endDate: string) {
  return useQuery({
    queryKey: ["escalation-funnel", startDate, endDate],
    queryFn: () =>
      tauriInvoke<EscalationFunnelEntry[]>("get_escalation_funnel", {
        startDate,
        endDate,
      }),
    enabled: !!startDate && !!endDate,
    staleTime: 30000,
  });
}
