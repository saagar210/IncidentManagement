export interface SlaDefinition {
  id: string;
  name: string;
  priority: string;
  response_time_minutes: number;
  resolve_time_minutes: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateSlaDefinitionRequest {
  name: string;
  priority: string;
  response_time_minutes: number;
  resolve_time_minutes: number;
}

export interface UpdateSlaDefinitionRequest {
  name?: string;
  priority?: string;
  response_time_minutes?: number;
  resolve_time_minutes?: number;
  is_active?: boolean;
}

export interface SlaStatus {
  priority: string;
  response_target_minutes: number | null;
  resolve_target_minutes: number | null;
  response_elapsed_minutes: number | null;
  resolve_elapsed_minutes: number | null;
  response_breached: boolean;
  resolve_breached: boolean;
}
