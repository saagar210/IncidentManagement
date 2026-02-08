import { useMutation } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { IncidentFilters } from "@/types/incident";

export function useExportCsv() {
  return useMutation({
    mutationFn: (filters: IncidentFilters) =>
      tauriInvoke<string>("export_incidents_csv", {
        filtersJson: JSON.stringify(filters),
      }),
  });
}

export function useExportJson() {
  return useMutation({
    mutationFn: (filters: IncidentFilters) =>
      tauriInvoke<string>("export_incidents_json", {
        filtersJson: JSON.stringify(filters),
      }),
  });
}
