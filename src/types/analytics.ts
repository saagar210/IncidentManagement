export interface SavedFilter {
  id: string;
  name: string;
  filters: string; // JSON string
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateSavedFilterRequest {
  name: string;
  filters: string;
  is_default?: boolean;
}

export interface UpdateSavedFilterRequest {
  name?: string;
  filters?: string;
  is_default?: boolean;
}

export interface BacklogAgingBucket {
  label: string;
  count: number;
}

export interface ServiceReliabilityScore {
  service_id: string;
  service_name: string;
  incident_count: number;
  mttr_minutes: number;
  mttr_formatted: string;
  sla_compliance_pct: number;
}

export interface EscalationFunnelEntry {
  severity: string;
  count: number;
  percentage: number;
}
