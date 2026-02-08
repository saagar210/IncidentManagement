export interface Attachment {
  id: string;
  incident_id: string;
  filename: string;
  file_path: string;
  mime_type: string;
  size_bytes: number;
  created_at: string;
}
