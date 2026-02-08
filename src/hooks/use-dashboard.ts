import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { DayCount, HourCount } from "@/types/metrics";

export function useIncidentHeatmap(startDate: string, endDate: string) {
  return useQuery({
    queryKey: ["heatmap", startDate, endDate],
    queryFn: () =>
      tauriInvoke<DayCount[]>("get_incident_heatmap", {
        startDate,
        endDate,
      }),
    enabled: startDate.length > 0 && endDate.length > 0,
    staleTime: 30000,
  });
}

export function useIncidentByHour(
  startDate: string | null,
  endDate: string | null
) {
  return useQuery({
    queryKey: ["by-hour", startDate, endDate],
    queryFn: () =>
      tauriInvoke<HourCount[]>("get_incident_by_hour", {
        startDate,
        endDate,
      }),
    enabled: !!startDate && !!endDate,
    staleTime: 30000,
  });
}

export function useDashboardConfig() {
  return useQuery({
    queryKey: ["dashboard-config"],
    queryFn: async () => {
      const raw = await tauriInvoke<string | null>("get_setting", {
        key: "dashboard_card_config",
      });
      if (!raw) return defaultConfig();
      try {
        return JSON.parse(raw) as DashboardCardConfig;
      } catch {
        return defaultConfig();
      }
    },
    staleTime: Infinity,
  });
}

export function useUpdateDashboardConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (config: DashboardCardConfig) => {
      await tauriInvoke<void>("set_setting", {
        key: "dashboard_card_config",
        value: JSON.stringify(config),
      });
      return config;
    },
    onSuccess: (config) => {
      queryClient.setQueryData(["dashboard-config"], config);
      queryClient.invalidateQueries({ queryKey: ["dashboard-config"] });
    },
  });
}

export interface DashboardCardConfig {
  mttr: boolean;
  mtta: boolean;
  recurrence_rate: boolean;
  avg_tickets: boolean;
  by_severity: boolean;
  by_service: boolean;
  heatmap: boolean;
  hour_histogram: boolean;
  trends: boolean;
  timeline: boolean;
}

function defaultConfig(): DashboardCardConfig {
  return {
    mttr: true,
    mtta: true,
    recurrence_rate: true,
    avg_tickets: true,
    by_severity: true,
    by_service: true,
    heatmap: true,
    hour_histogram: true,
    trends: true,
    timeline: false,
  };
}
