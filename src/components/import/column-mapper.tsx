import { Select } from "@/components/ui/select";

const INCIDENT_FIELDS = [
  { value: "", label: "-- Skip --" },
  { value: "title", label: "Title" },
  { value: "service", label: "Service" },
  { value: "severity", label: "Severity" },
  { value: "impact", label: "Impact" },
  { value: "status", label: "Status" },
  { value: "started_at", label: "Started At" },
  { value: "detected_at", label: "Detected At" },
  { value: "responded_at", label: "Responded At" },
  { value: "resolved_at", label: "Resolved At" },
  { value: "root_cause", label: "Root Cause" },
  { value: "resolution", label: "Resolution" },
  { value: "tickets_submitted", label: "Tickets Submitted" },
  { value: "affected_users", label: "Affected Users" },
  { value: "is_recurring", label: "Is Recurring" },
  { value: "lessons_learned", label: "Lessons Learned" },
  { value: "external_ref", label: "External Ref" },
  { value: "notes", label: "Notes" },
] as const;

/** Common auto-detect patterns for CSV column names */
const AUTO_DETECT_MAP: Record<string, string> = {
  title: "title",
  incidenttitle: "title",
  name: "title",
  summary: "title",
  service: "service",
  servicename: "service",
  system: "service",
  application: "service",
  severity: "severity",
  sev: "severity",
  impact: "impact",
  status: "status",
  state: "status",
  startedat: "started_at",
  startdate: "started_at",
  start: "started_at",
  detectedat: "detected_at",
  detected: "detected_at",
  respondedat: "responded_at",
  responded: "responded_at",
  resolvedat: "resolved_at",
  resolved: "resolved_at",
  enddate: "resolved_at",
  rootcause: "root_cause",
  cause: "root_cause",
  resolution: "resolution",
  fix: "resolution",
  tickets: "tickets_submitted",
  ticketssubmitted: "tickets_submitted",
  affectedusers: "affected_users",
  users: "affected_users",
  recurring: "is_recurring",
  isrecurring: "is_recurring",
  lessonslearned: "lessons_learned",
  lessons: "lessons_learned",
  externalref: "external_ref",
  reference: "external_ref",
  ref: "external_ref",
  jira: "external_ref",
  notes: "notes",
  comments: "notes",
  description: "notes",
};

export function autoDetectMappings(
  csvColumns: string[]
): Record<string, string> {
  const result: Record<string, string> = {};
  const usedFields = new Set<string>();

  for (const col of csvColumns) {
    const normalized = col.toLowerCase().replace(/[\s_-]/g, "");
    const field = AUTO_DETECT_MAP[normalized];
    if (field && !usedFields.has(field)) {
      result[col] = field;
      usedFields.add(field);
    }
  }

  return result;
}

interface ColumnMapperProps {
  csvColumns: string[];
  mappings: Record<string, string>;
  onMappingChange: (csvColumn: string, incidentField: string) => void;
}

export function ColumnMapper({
  csvColumns,
  mappings,
  onMappingChange,
}: ColumnMapperProps) {
  // Track which fields are already mapped to prevent duplicates
  const usedFields = new Set(Object.values(mappings).filter(Boolean));

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-[1fr,auto,1fr] gap-2 items-center text-sm font-medium text-muted-foreground">
        <span>CSV Column</span>
        <span></span>
        <span>Maps To</span>
      </div>
      {csvColumns.map((col) => {
        const currentMapping = mappings[col] || "";
        return (
          <div
            key={col}
            className="grid grid-cols-[1fr,auto,1fr] gap-2 items-center"
          >
            <div className="truncate rounded bg-muted px-3 py-2 text-sm font-mono">
              {col}
            </div>
            <span className="text-muted-foreground text-sm">-&gt;</span>
            <Select
              value={currentMapping}
              onChange={(e) => onMappingChange(col, e.target.value)}
            >
              {INCIDENT_FIELDS.map((field) => {
                const isUsed =
                  field.value !== "" &&
                  field.value !== currentMapping &&
                  usedFields.has(field.value);
                return (
                  <option
                    key={field.value}
                    value={field.value}
                    disabled={isUsed}
                  >
                    {field.label}
                    {isUsed ? " (already mapped)" : ""}
                  </option>
                );
              })}
            </Select>
          </div>
        );
      })}
    </div>
  );
}
