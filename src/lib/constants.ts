export const SEVERITY_LEVELS = ["Critical", "High", "Medium", "Low"] as const;
export type SeverityLevel = (typeof SEVERITY_LEVELS)[number];

export const IMPACT_LEVELS = ["Critical", "High", "Medium", "Low"] as const;
export type ImpactLevel = (typeof IMPACT_LEVELS)[number];

export const STATUS_OPTIONS = [
  "Active",
  "Acknowledged",
  "Monitoring",
  "Resolved",
  "Post-Mortem",
] as const;
export type StatusOption = (typeof STATUS_OPTIONS)[number];

export const PRIORITY_LEVELS = ["P0", "P1", "P2", "P3", "P4"] as const;
export type PriorityLevel = (typeof PRIORITY_LEVELS)[number];

export const SERVICE_CATEGORIES = [
  "Communication",
  "Infrastructure",
  "Development",
  "Productivity",
  "Security",
  "Other",
] as const;
export type ServiceCategory = (typeof SERVICE_CATEGORIES)[number];

export const ACTION_ITEM_STATUSES = ["Open", "In-Progress", "Done"] as const;
export type ActionItemStatus = (typeof ACTION_ITEM_STATUSES)[number];

export const SERVICE_TIERS = ["T1", "T2", "T3", "T4"] as const;
export type ServiceTier = (typeof SERVICE_TIERS)[number];

export const TIER_LABELS: Record<ServiceTier, string> = {
  T1: "T1 - Critical",
  T2: "T2 - Important",
  T3: "T3 - Standard",
  T4: "T4 - Best Effort",
};

export const TIER_COLORS: Record<ServiceTier, string> = {
  T1: "bg-red-500/10 text-red-500 border-red-500/20",
  T2: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  T3: "bg-blue-500/10 text-blue-500 border-blue-500/20",
  T4: "bg-zinc-500/10 text-zinc-500 border-zinc-500/20",
};

export const DEPENDENCY_TYPES = ["runtime", "build", "data", "optional"] as const;
export type DependencyType = (typeof DEPENDENCY_TYPES)[number];

export const INCIDENT_ROLES = [
  "Incident Commander",
  "Communications Lead",
  "Technical Lead",
  "Scribe",
  "SME",
] as const;
export type IncidentRoleType = (typeof INCIDENT_ROLES)[number];

export const SEVERITY_COLORS: Record<SeverityLevel, string> = {
  Critical: "bg-red-500/10 text-red-500 border-red-500/20",
  High: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  Medium: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  Low: "bg-blue-500/10 text-blue-500 border-blue-500/20",
};

export const IMPACT_COLORS = SEVERITY_COLORS;

export const PRIORITY_COLORS: Record<PriorityLevel, string> = {
  P0: "bg-red-500/10 text-red-500 border-red-500/20 animate-pulse",
  P1: "bg-red-500/10 text-red-500 border-red-500/20",
  P2: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  P3: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  P4: "bg-zinc-500/10 text-zinc-500 border-zinc-500/20",
};

export const STATUS_COLORS: Record<StatusOption, string> = {
  Active: "bg-red-500/10 text-red-500 border-red-500/20",
  Acknowledged: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  Monitoring: "bg-yellow-500/10 text-yellow-600 border-yellow-500/20",
  Resolved: "bg-green-500/10 text-green-600 border-green-500/20",
  "Post-Mortem": "bg-blue-500/10 text-blue-500 border-blue-500/20",
};

export const STATUS_TRANSITIONS: Record<StatusOption, StatusOption[]> = {
  Active: ["Acknowledged", "Monitoring", "Resolved"],
  Acknowledged: ["Active", "Monitoring", "Resolved"],
  Monitoring: ["Active", "Acknowledged", "Resolved"],
  Resolved: ["Active", "Post-Mortem"],
  "Post-Mortem": ["Active"],
};

export const CHART_COLORS = {
  severity: {
    Critical: "hsl(0, 84%, 60%)",
    High: "hsl(25, 95%, 53%)",
    Medium: "hsl(45, 93%, 47%)",
    Low: "hsl(217, 91%, 60%)",
  },
  status: {
    Active: "hsl(0, 84%, 60%)",
    Acknowledged: "hsl(25, 95%, 53%)",
    Monitoring: "hsl(45, 93%, 47%)",
    Resolved: "hsl(142, 76%, 36%)",
    "Post-Mortem": "hsl(217, 91%, 60%)",
  },
};
