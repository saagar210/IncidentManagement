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

export interface ReportConfig {
  quarter_id: string | null;
  fiscal_year: number | null;
  title: string;
  introduction: string;
  sections: ReportSections;
  chart_images: Record<string, string>;
}

export interface DiscussionPoint {
  text: string;
  trigger: string;
  severity: string;
}
