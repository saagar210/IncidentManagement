export interface TimelineEvent {
  id: string;
  incident_id: string;
  occurred_at: string;
  source: string;
  message: string;
  actor: string;
  created_at: string;
}

export interface CreateTimelineEventRequest {
  incident_id: string;
  occurred_at: string;
  source?: string;
  message: string;
  actor?: string;
}

export interface TimelineImportResult {
  created: number;
  skipped: number;
  errors: string[];
}

