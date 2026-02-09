export interface FieldProvenance {
  id: string;
  entity_type: string;
  entity_id: string;
  field_name: string;
  source_type: string;
  source_ref: string;
  source_version: string;
  input_hash: string;
  meta_json: string;
  recorded_at: string;
}

