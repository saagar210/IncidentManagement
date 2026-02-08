export interface Incident {
  id: string;
  title: string;
  service_id: string;
  service_name: string;
  severity: string;
  impact: string;
  priority: string;
  status: string;
  started_at: string;
  detected_at: string;
  responded_at: string | null;
  resolved_at: string | null;
  duration_minutes: number | null;
  root_cause: string;
  resolution: string;
  tickets_submitted: number;
  affected_users: number;
  is_recurring: boolean;
  recurrence_of: string | null;
  lessons_learned: string;
  action_items: string;
  external_ref: string;
  notes: string;
  created_at: string;
  updated_at: string;
}

export interface CreateIncidentRequest {
  title: string;
  service_id: string;
  severity: string;
  impact: string;
  status: string;
  started_at: string;
  detected_at: string;
  responded_at?: string | null;
  resolved_at?: string | null;
  root_cause?: string;
  resolution?: string;
  tickets_submitted?: number;
  affected_users?: number;
  is_recurring?: boolean;
  recurrence_of?: string | null;
  lessons_learned?: string;
  action_items?: string;
  external_ref?: string;
  notes?: string;
}

export interface UpdateIncidentRequest {
  title?: string;
  service_id?: string;
  severity?: string;
  impact?: string;
  status?: string;
  started_at?: string;
  detected_at?: string;
  responded_at?: string | null;
  resolved_at?: string | null;
  root_cause?: string;
  resolution?: string;
  tickets_submitted?: number;
  affected_users?: number;
  is_recurring?: boolean;
  recurrence_of?: string | null;
  lessons_learned?: string;
  action_items?: string;
  external_ref?: string;
  notes?: string;
}

export interface IncidentFilters {
  service_id?: string;
  severity?: string;
  impact?: string;
  status?: string;
  quarter_id?: string;
  date_from?: string;
  date_to?: string;
  sort_by?: string;
  sort_order?: string;
}

export interface ActionItem {
  id: string;
  incident_id: string;
  title: string;
  description: string;
  status: string;
  owner: string;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateActionItemRequest {
  incident_id: string;
  title: string;
  description?: string;
  status?: string;
  owner?: string;
  due_date?: string | null;
}

export interface UpdateActionItemRequest {
  title?: string;
  description?: string;
  status?: string;
  owner?: string;
  due_date?: string | null;
}

export interface Service {
  id: string;
  name: string;
  category: string;
  default_severity: string;
  default_impact: string;
  description: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateServiceRequest {
  name: string;
  category: string;
  default_severity: string;
  default_impact: string;
  description?: string;
}

export interface UpdateServiceRequest {
  name?: string;
  category?: string;
  default_severity?: string;
  default_impact?: string;
  description?: string;
  is_active?: boolean;
}

export interface QuarterConfig {
  id: string;
  fiscal_year: number;
  quarter_number: number;
  start_date: string;
  end_date: string;
  label: string;
  created_at: string;
}

export interface UpsertQuarterRequest {
  id?: string;
  fiscal_year: number;
  quarter_number: number;
  start_date: string;
  end_date: string;
  label: string;
}
