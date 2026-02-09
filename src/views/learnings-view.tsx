import { useState, useRef, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { BookOpen, Search, FileText } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useActiveServices } from "@/hooks/use-services";
import { usePirReviewInsights } from "@/hooks/use-pir-review";
import { tauriInvoke } from "@/lib/tauri";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { SavedFilterBar } from "@/components/incidents/saved-filter-bar";
import { SEVERITY_COLORS, SEVERITY_LEVELS, STATUS_OPTIONS } from "@/lib/constants";
import type { SeverityLevel } from "@/lib/constants";
import type { Incident } from "@/types/incident";
import type { IncidentFilters } from "@/types/incident";

const EMPTY_FILTERS: IncidentFilters = {};
const LEARNINGS_FILTERS_KEY = "learnings.filters.v1";

function pickLearningsFilters(filters: IncidentFilters): IncidentFilters {
  return {
    service_id: filters.service_id,
    severity: filters.severity,
    status: filters.status,
    tag: filters.tag,
  };
}

export function LearningsView() {
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [filters, setFilters] = useState<IncidentFilters>(EMPTY_FILTERS);
  const { data: services } = useActiveServices();
  const { data: insights } = usePirReviewInsights();
  const { data: allTags } = useQuery({
    queryKey: ["all-tags"],
    queryFn: () => tauriInvoke<string[]>("get_all_tags"),
  });

  useEffect(() => {
    // Best-effort persistence; if local storage is unavailable or corrupt, ignore.
    try {
      const raw = localStorage.getItem(LEARNINGS_FILTERS_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as unknown;
      if (parsed && typeof parsed === "object") {
        setFilters(parsed as IncidentFilters);
      }
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    const t = setTimeout(() => {
      try {
        localStorage.setItem(LEARNINGS_FILTERS_KEY, JSON.stringify(filters));
      } catch {
        // ignore
      }
    }, 250);
    return () => clearTimeout(t);
  }, [filters]);

  // Debounce search
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const handleSearch = (value: string) => {
    setSearchQuery(value);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      setDebouncedQuery(value);
    }, 300);
  };

  const { data: results, isLoading } = useQuery({
    queryKey: ["learnings-search", debouncedQuery, pickLearningsFilters(filters)],
    queryFn: () =>
      tauriInvoke<Incident[]>("search_incidents_filtered", {
        query: debouncedQuery,
        serviceId: filters.service_id ?? null,
        severity: filters.severity ?? null,
        status: filters.status ?? null,
        tag: filters.tag ?? null,
      }),
    enabled: debouncedQuery.length >= 2,
  });

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center gap-3">
        <BookOpen className="h-6 w-6 text-purple-500" />
        <h1 className="text-2xl font-semibold">Learnings Database</h1>
      </div>

      <p className="text-sm text-muted-foreground">
        Search across all incidents, root causes, resolutions, and lessons
        learned. Uses full-text search for fast, relevant results.
      </p>

      {/* Search */}
      <div className="relative max-w-xl">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search incidents, root causes, lessons learned..."
          value={searchQuery}
          onChange={(e) => handleSearch(e.target.value)}
          className="pl-9"
        />
      </div>

      {/* Filters + Saved Filters */}
      <div className="flex flex-wrap items-center gap-2">
        <Select
          value={filters.service_id ?? ""}
          onChange={(e) =>
            setFilters((f) => ({
              ...f,
              service_id: e.target.value || undefined,
            }))
          }
          className="w-56"
        >
          <option value="">All Services</option>
          {services?.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </Select>

        <Select
          value={filters.severity ?? ""}
          onChange={(e) =>
            setFilters((f) => ({
              ...f,
              severity: e.target.value || undefined,
            }))
          }
          className="w-40"
        >
          <option value="">All Severity</option>
          {SEVERITY_LEVELS.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </Select>

        <Select
          value={filters.status ?? ""}
          onChange={(e) =>
            setFilters((f) => ({
              ...f,
              status: e.target.value || undefined,
            }))
          }
          className="w-44"
        >
          <option value="">All Status</option>
          {STATUS_OPTIONS.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </Select>

        <Select
          value={filters.tag ?? ""}
          onChange={(e) =>
            setFilters((f) => ({
              ...f,
              tag: e.target.value || undefined,
            }))
          }
          className="w-44"
        >
          <option value="">All Tags</option>
          {allTags?.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </Select>
      </div>

      <SavedFilterBar
        currentFilters={filters}
        onApplyFilter={(f) => setFilters(pickLearningsFilters(f))}
      />

      {insights && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Review Insights</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-3 sm:grid-cols-3">
            <div>
              <div className="text-xs font-medium text-muted-foreground">
                Top Root Factor Categories
              </div>
              <ul className="mt-1 space-y-1 text-sm">
                {insights.top_factor_categories.map((x) => (
                  <li key={x.label} className="flex justify-between gap-3">
                    <span className="truncate">{x.label}</span>
                    <span className="tabular-nums text-muted-foreground">
                      {x.count}
                    </span>
                  </li>
                ))}
                {insights.top_factor_categories.length === 0 && (
                  <li className="text-muted-foreground">No data</li>
                )}
              </ul>
            </div>

            <div>
              <div className="text-xs font-medium text-muted-foreground">
                Top Root Factor Descriptions
              </div>
              <ul className="mt-1 space-y-1 text-sm">
                {insights.top_factor_descriptions.map((x) => (
                  <li key={x.label} className="flex justify-between gap-3">
                    <span className="truncate">{x.label}</span>
                    <span className="tabular-nums text-muted-foreground">
                      {x.count}
                    </span>
                  </li>
                ))}
                {insights.top_factor_descriptions.length === 0 && (
                  <li className="text-muted-foreground">No data</li>
                )}
              </ul>
            </div>

            <div>
              <div className="text-xs font-medium text-muted-foreground">
                External Root, No Action Items Justified
              </div>
              <div className="mt-1 text-2xl font-semibold tabular-nums">
                {insights.external_root_no_action_items_justified}
              </div>
              <div className="text-xs text-muted-foreground">
                Incidents where the root factor was External and no actions were
                justified.
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Results */}
      {isLoading && (
        <p className="text-sm text-muted-foreground">Searching...</p>
      )}

      {debouncedQuery.length >= 2 && results && results.length === 0 && (
        <p className="text-sm text-muted-foreground">
          No results found for &ldquo;{debouncedQuery}&rdquo;
        </p>
      )}

      {results && results.length > 0 && (
        <div className="space-y-3">
          <p className="text-sm text-muted-foreground">
            {results.length} result{results.length !== 1 ? "s" : ""} found
          </p>
          {results.map((inc) => (
            <Card
              key={inc.id}
              className="cursor-pointer transition-colors hover:bg-accent/50"
              onClick={() => navigate(`/incidents/${inc.id}`)}
            >
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="flex items-center gap-2 text-sm">
                    <FileText className="h-4 w-4 text-muted-foreground" />
                    {inc.title}
                  </CardTitle>
                  <div className="flex items-center gap-2">
                    <Badge
                      variant="outline"
                      className={
                        SEVERITY_COLORS[inc.severity as SeverityLevel] ?? ""
                      }
                    >
                      {inc.severity}
                    </Badge>
                    <Badge variant="outline">{inc.status}</Badge>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground">
                  {inc.service_name}
                </p>
              </CardHeader>
              <CardContent className="space-y-2">
                {inc.root_cause && (
                  <div>
                    <span className="text-xs font-medium text-muted-foreground">
                      Root Cause:
                    </span>
                    <p className="line-clamp-2 text-sm">{inc.root_cause}</p>
                  </div>
                )}
                {inc.lessons_learned && (
                  <div>
                    <span className="text-xs font-medium text-muted-foreground">
                      Lessons:
                    </span>
                    <p className="line-clamp-2 text-sm">{inc.lessons_learned}</p>
                  </div>
                )}
                {inc.resolution && (
                  <div>
                    <span className="text-xs font-medium text-muted-foreground">
                      Resolution:
                    </span>
                    <p className="line-clamp-2 text-sm">{inc.resolution}</p>
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {debouncedQuery.length < 2 && (
        <div className="rounded-lg border border-dashed p-8 text-center">
          <BookOpen className="mx-auto mb-3 h-10 w-10 text-muted-foreground/50" />
          <p className="text-sm text-muted-foreground">
            Type at least 2 characters to search across all incidents and
            learnings.
          </p>
        </div>
      )}
    </div>
  );
}
