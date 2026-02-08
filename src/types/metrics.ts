export interface MetricResult {
  value: number;
  previous_value: number | null;
  trend: "Up" | "Down" | "Flat" | "NoData";
  formatted_value: string;
}

export interface CategoryCount {
  category: string;
  count: number;
  previous_count: number | null;
}

export interface ServiceDowntime {
  service_id: string;
  service_name: string;
  total_minutes: number;
  formatted: string;
}

export interface QuarterlyTrends {
  quarters: string[];
  mttr: number[];
  mtta: number[];
  incident_count: number[];
  recurrence_rate: number[];
  avg_tickets: number[];
}

export interface DashboardData {
  mttr: MetricResult;
  mtta: MetricResult;
  recurrence_rate: MetricResult;
  avg_tickets: MetricResult;
  by_severity: CategoryCount[];
  by_impact: CategoryCount[];
  by_service: CategoryCount[];
  downtime_by_service: ServiceDowntime[];
  trends: QuarterlyTrends;
  total_incidents: number;
  period_label: string;
}

export interface MetricFilters {
  service_ids?: string[];
  min_severity?: string;
  min_impact?: string;
}

export interface DayCount {
  day: string;
  count: number;
}

export interface HourCount {
  hour: number;
  count: number;
}
