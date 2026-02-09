export type EnrichmentJobStatus = "queued" | "running" | "succeeded" | "failed";

export interface EnrichmentJob {
  id: string;
  job_type: string;
  entity_type: string;
  entity_id: string;
  status: EnrichmentJobStatus | string;
  input_hash: string;
  output_json: string;
  model_id: string;
  prompt_version: string;
  error: string;
  created_at: string;
  completed_at: string | null;
}

export interface IncidentEnrichment {
  incident_id: string;
  executive_summary: string;
  last_job_id: string | null;
  generated_by: string;
  updated_at: string;
}

