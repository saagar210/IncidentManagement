export interface AuditEntry {
  id: string;
  entity_type: string;
  entity_id: string;
  action: string;
  summary: string;
  details: string;
  created_at: string;
}

export interface AuditFilters {
  entity_type?: string;
  entity_id?: string;
  action?: string;
  limit?: number;
}

export interface NotificationSummary {
  active_incidents: number;
  overdue_action_items: number;
  sla_breaches: number;
  recent_audit_count: number;
}
