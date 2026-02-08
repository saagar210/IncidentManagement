export interface AiStatus {
  available: boolean;
  base_url: string;
  primary_model: string;
  fast_model: string;
}

export interface SimilarIncident {
  id: string;
  title: string;
  service_name: string;
  severity: string;
  status: string;
  rank: number;
}
