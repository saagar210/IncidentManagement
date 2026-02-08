export interface ChecklistTemplate {
  id: string;
  name: string;
  service_id: string | null;
  incident_type: string | null;
  is_active: boolean;
  items: ChecklistTemplateItem[];
  created_at: string;
  updated_at: string;
}

export interface ChecklistTemplateItem {
  id: string;
  template_id: string;
  label: string;
  sort_order: number;
}

export interface CreateChecklistTemplateRequest {
  name: string;
  service_id?: string;
  incident_type?: string;
  items: string[];
}

export interface UpdateChecklistTemplateRequest {
  name?: string;
  is_active?: boolean;
  items?: string[];
}

export interface IncidentChecklist {
  id: string;
  incident_id: string;
  template_id: string | null;
  name: string;
  items: ChecklistItem[];
  created_at: string;
}

export interface ChecklistItem {
  id: string;
  checklist_id: string;
  template_item_id: string | null;
  label: string;
  is_checked: boolean;
  checked_at: string | null;
  checked_by: string | null;
  sort_order: number;
}

export interface CreateIncidentChecklistRequest {
  incident_id: string;
  template_id?: string;
  name?: string;
  items?: string[];
}

export interface ToggleChecklistItemRequest {
  checked_by?: string;
}
