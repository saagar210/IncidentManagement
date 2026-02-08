import { useState, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { BookOpen, Search, FileText } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { SEVERITY_COLORS } from "@/lib/constants";
import type { SeverityLevel } from "@/lib/constants";
import type { Incident } from "@/types/incident";

export function LearningsView() {
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");

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
    queryKey: ["learnings-search", debouncedQuery],
    queryFn: () =>
      tauriInvoke<Incident[]>("search_incidents", {
        query: debouncedQuery,
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
