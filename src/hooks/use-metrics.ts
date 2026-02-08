import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { DashboardData, MetricFilters } from "@/types/metrics";

const EMPTY_FILTERS: MetricFilters = {};

export function useDashboardData(
  quarterId: string | null,
  filters: MetricFilters = EMPTY_FILTERS
) {
  return useQuery({
    queryKey: ["dashboard", quarterId, filters],
    queryFn: () =>
      tauriInvoke<DashboardData>("get_dashboard_data", {
        quarterId,
        filters,
      }),
    staleTime: 30000,
  });
}
