export interface CustomFieldDefinition {
  id: string;
  name: string;
  field_type: "text" | "number" | "select";
  options: string;
  display_order: number;
  created_at: string;
  updated_at: string;
}

export interface CreateCustomFieldRequest {
  name: string;
  field_type: "text" | "number" | "select";
  options?: string;
  display_order?: number;
}

export interface UpdateCustomFieldRequest {
  name?: string;
  field_type?: "text" | "number" | "select";
  options?: string;
  display_order?: number;
}

export interface CustomFieldValue {
  incident_id: string;
  field_id: string;
  value: string;
}
