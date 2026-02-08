export interface IncidentRole {
  id: string;
  incident_id: string;
  role: string;
  assignee: string;
  is_primary: boolean;
  assigned_at: string;
  unassigned_at: string | null;
}

export interface AssignRoleRequest {
  incident_id: string;
  role: string;
  assignee: string;
  is_primary?: boolean;
}
