export interface ContributingFactor {
  id: string;
  incident_id: string;
  category: ContributingFactorCategory;
  description: string;
  is_root: boolean;
  created_at: string;
}

export type ContributingFactorCategory =
  | "Process"
  | "Tooling"
  | "Communication"
  | "Human Factors"
  | "External";

export const CONTRIBUTING_FACTOR_CATEGORIES: ContributingFactorCategory[] = [
  "Process",
  "Tooling",
  "Communication",
  "Human Factors",
  "External",
];

export interface CreateContributingFactorRequest {
  incident_id: string;
  category: ContributingFactorCategory;
  description: string;
  is_root?: boolean;
}

export interface PostmortemTemplate {
  id: string;
  name: string;
  incident_type: string;
  template_content: string;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface Postmortem {
  id: string;
  incident_id: string;
  template_id: string | null;
  content: string;
  status: "draft" | "review" | "final";
  reminder_at: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreatePostmortemRequest {
  incident_id: string;
  template_id?: string;
  content?: string;
}

export interface UpdatePostmortemRequest {
  content?: string;
  status?: "draft" | "review" | "final";
  reminder_at?: string;
}
