export interface ReportSections {
  executive_summary: boolean;
  metrics_overview: boolean;
  incident_timeline: boolean;
  incident_breakdowns: boolean;
  service_reliability: boolean;
  qoq_comparison: boolean;
  discussion_points: boolean;
  action_items: boolean;
}

export type ReportFormat = "docx" | "pdf";

export interface ReportConfig {
  quarter_id: string | null;
  fiscal_year: number | null;
  title: string;
  introduction: string;
  sections: ReportSections;
  chart_images: Record<string, string>;
  format: ReportFormat;
}

export interface DiscussionPoint {
  text: string;
  trigger: string;
  severity: string;
}

export interface ReportHistoryEntry {
  id: string;
  title: string;
  quarter_id: string | null;
  format: string;
  generated_at: string;
  file_path: string;
  config_json: string;
  file_size_bytes: number | null;
}
