import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import type { EnrichmentJob, IncidentEnrichment } from "@/types/enrichments";

export function useIncidentEnrichment(incidentId: string | null) {
  return useQuery({
    queryKey: ["incident-enrichment", incidentId],
    queryFn: () => tauriInvoke<IncidentEnrichment | null>("get_incident_enrichment", { incidentId }),
    enabled: !!incidentId,
  });
}

export function useRunIncidentEnrichment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: { job_type: string; incident_id: string }) =>
      tauriInvoke<EnrichmentJob>("run_incident_enrichment", { req }),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["incident-enrichment", vars.incident_id] });
    },
  });
}

export function useAcceptEnrichmentJob() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (jobId: string) =>
      tauriInvoke<void>("accept_enrichment_job", { req: { job_id: jobId } }),
    onSuccess: () => {
      // Broad invalidation: accept may materialize stakeholder updates/postmortems/etc.
      qc.invalidateQueries();
    },
  });
}

