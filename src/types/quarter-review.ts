export interface ReadinessFinding {
  rule_key: string;
  severity: "critical" | "warning" | string;
  message: string;
  incident_ids: string[];
  remediation: string;
}

export interface QuarterReadinessReport {
  quarter_id: string;
  quarter_label: string;
  total_incidents: number;
  ready_incidents: number;
  needs_attention_incidents: number;
  findings: ReadinessFinding[];
}

export interface MetricDefinition {
  key: string;
  name: string;
  definition: string;
  calculation: string;
  inclusion: string;
}

export interface QuarterOverride {
  id: string;
  quarter_id: string;
  rule_key: string;
  incident_id: string;
  reason: string;
  approved_by: string;
  created_at: string;
}

export interface QuarterFinalization {
  quarter_id: string;
  finalized_at: string;
  finalized_by: string;
  snapshot_id: string;
  inputs_hash: string;
  notes: string;
}

export interface QuarterFinalizationStatus {
  quarter_id: string;
  finalized: boolean;
  finalization: QuarterFinalization | null;
  readiness: QuarterReadinessReport;
  overrides: QuarterOverride[];
  snapshot_inputs_hash: string | null;
  current_inputs_hash: string;
  facts_changed_since_finalization: boolean;
}
