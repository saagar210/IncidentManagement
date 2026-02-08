import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Plus, Search, ChevronLeft, ChevronRight } from "lucide-react";
import { format } from "date-fns";
import { useIncidents } from "@/hooks/use-incidents";
import { useActiveServices } from "@/hooks/use-services";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Select } from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import {
  SEVERITY_LEVELS,
  STATUS_OPTIONS,
  SEVERITY_COLORS,
  STATUS_COLORS,
  IMPACT_COLORS,
  PRIORITY_COLORS,
} from "@/lib/constants";
import type { IncidentFilters, Incident } from "@/types/incident";
import type { SeverityLevel, StatusOption, PriorityLevel } from "@/lib/constants";

const PAGE_SIZE = 25;

function useDebounce(value: string, delay: number): string {
  const [debouncedValue, setDebouncedValue] = useState(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);

  return debouncedValue;
}

function formatDuration(minutes: number | null): string {
  if (minutes === null) return "--";
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours < 24) return `${hours}h ${mins}m`;
  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return `${days}d ${remainingHours}h`;
}

function formatDate(dateStr: string): string {
  try {
    return format(new Date(dateStr), "MMM d, yyyy HH:mm");
  } catch {
    return dateStr;
  }
}

export function IncidentsView() {
  const navigate = useNavigate();
  const [searchText, setSearchText] = useState("");
  const [page, setPage] = useState(0);
  const [filters, setFilters] = useState<IncidentFilters>({});

  const debouncedSearch = useDebounce(searchText, 300);

  const { data: services } = useActiveServices();
  const { data: incidents, isLoading } = useIncidents(filters);

  const filteredIncidents = useMemo(() => {
    if (!incidents) return [];
    if (!debouncedSearch) return incidents;
    const lower = debouncedSearch.toLowerCase();
    return incidents.filter(
      (inc: Incident) =>
        inc.title.toLowerCase().includes(lower) ||
        inc.service_name.toLowerCase().includes(lower) ||
        inc.external_ref?.toLowerCase().includes(lower)
    );
  }, [incidents, debouncedSearch]);

  const totalPages = Math.max(1, Math.ceil(filteredIncidents.length / PAGE_SIZE));
  const pagedIncidents = filteredIncidents.slice(
    page * PAGE_SIZE,
    (page + 1) * PAGE_SIZE
  );

  const updateFilter = useCallback(
    (key: keyof IncidentFilters, value: string) => {
      setPage(0);
      setFilters((prev) => {
        const next = { ...prev };
        if (value) {
          (next as Record<string, string>)[key] = value;
        } else {
          delete (next as Record<string, string | undefined>)[key];
        }
        return next;
      });
    },
    []
  );

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Incidents</h1>
        <Button onClick={() => navigate("/incidents/new")}>
          <Plus className="h-4 w-4" />
          New Incident
        </Button>
      </div>

      {/* Filter bar */}
      <div className="flex flex-wrap items-center gap-3">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search incidents..."
            value={searchText}
            onChange={(e) => {
              setSearchText(e.target.value);
              setPage(0);
            }}
            className="pl-9"
          />
        </div>
        <Select
          value={filters.service_id ?? ""}
          onChange={(e) => updateFilter("service_id", e.target.value)}
          className="w-40"
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
          onChange={(e) => updateFilter("severity", e.target.value)}
          className="w-32"
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
          onChange={(e) => updateFilter("status", e.target.value)}
          className="w-36"
        >
          <option value="">All Status</option>
          {STATUS_OPTIONS.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </Select>
      </div>

      {/* Table */}
      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 8 }).map((_, i) => (
            <Skeleton key={i} className="h-10 w-full" />
          ))}
        </div>
      ) : pagedIncidents.length === 0 ? (
        <div className="flex h-64 items-center justify-center text-muted-foreground">
          <p>No incidents found. Create your first incident to get started.</p>
        </div>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Title</TableHead>
              <TableHead>Service</TableHead>
              <TableHead>Severity</TableHead>
              <TableHead>Impact</TableHead>
              <TableHead>Priority</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Started At</TableHead>
              <TableHead>Duration</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {pagedIncidents.map((incident: Incident) => (
              <TableRow
                key={incident.id}
                className="cursor-pointer"
                onClick={() => navigate(`/incidents/${incident.id}`)}
              >
                <TableCell className="font-medium max-w-[250px] truncate">
                  {incident.title}
                </TableCell>
                <TableCell>{incident.service_name}</TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={
                      SEVERITY_COLORS[incident.severity as SeverityLevel] ?? ""
                    }
                  >
                    {incident.severity}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={
                      IMPACT_COLORS[incident.impact as SeverityLevel] ?? ""
                    }
                  >
                    {incident.impact}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={
                      PRIORITY_COLORS[incident.priority as PriorityLevel] ?? ""
                    }
                  >
                    {incident.priority}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={
                      STATUS_COLORS[incident.status as StatusOption] ?? ""
                    }
                  >
                    {incident.status}
                  </Badge>
                </TableCell>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatDate(incident.started_at)}
                </TableCell>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatDuration(incident.duration_minutes)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      {/* Pagination */}
      {filteredIncidents.length > PAGE_SIZE && (
        <div className="flex items-center justify-between border-t pt-4">
          <p className="text-sm text-muted-foreground">
            Showing {page * PAGE_SIZE + 1}&ndash;
            {Math.min((page + 1) * PAGE_SIZE, filteredIncidents.length)} of{" "}
            {filteredIncidents.length}
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={page === 0}
              onClick={() => setPage((p) => p - 1)}
            >
              <ChevronLeft className="h-4 w-4" />
              Prev
            </Button>
            <span className="text-sm text-muted-foreground">
              {page + 1} / {totalPages}
            </span>
            <Button
              variant="outline"
              size="sm"
              disabled={page >= totalPages - 1}
              onClick={() => setPage((p) => p + 1)}
            >
              Next
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
