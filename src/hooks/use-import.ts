import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { tauriInvoke } from "@/lib/tauri";

// ---- Types ----

export interface ColumnMapping {
  mappings: Record<string, string>;
  default_values: Record<string, string>;
}

export interface PreviewRow {
  title: string;
  service_name: string;
  severity: string;
  impact: string;
  status: string;
  started_at: string;
  detected_at: string;
  row_status: "ready" | "warning" | "error";
  messages: string[];
}

export interface ImportPreview {
  incidents: PreviewRow[];
  warnings: { row: number; field: string; message: string }[];
  error_count: number;
  ready_count: number;
  warning_count: number;
}

export interface ImportResult {
  created: number;
  updated: number;
  skipped: number;
  errors: string[];
}

export interface ImportTemplate {
  id: string;
  name: string;
  column_mapping: string;
  source: string;
  schema_version: number;
  created_at: string;
  updated_at: string;
}

export interface BackupImportResult {
  services: number;
  incidents: number;
  action_items: number;
  quarter_configs: number;
  custom_field_definitions: number;
  custom_field_values: number;
  settings: number;
  errors: string[];
}

// ---- Hooks ----

export function useParseCSVHeaders() {
  return useMutation({
    mutationFn: (filePath: string) =>
      tauriInvoke<string[]>("parse_csv_headers", { filePath }),
  });
}

export function usePreviewImport() {
  return useMutation({
    mutationFn: ({
      filePath,
      mapping,
    }: {
      filePath: string;
      mapping: ColumnMapping;
    }) =>
      tauriInvoke<ImportPreview>("preview_csv_import", { filePath, mapping }),
  });
}

export function useExecuteImport() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      filePath,
      mapping,
    }: {
      filePath: string;
      mapping: ColumnMapping;
    }) =>
      tauriInvoke<ImportResult>("execute_csv_import", { filePath, mapping }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["incidents"] });
    },
  });
}

export function useImportTemplates() {
  return useQuery({
    queryKey: ["import-templates"],
    queryFn: () => tauriInvoke<ImportTemplate[]>("list_import_templates"),
  });
}

export function useSaveTemplate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      name,
      columnMapping,
    }: {
      name: string;
      columnMapping: string;
    }) =>
      tauriInvoke<ImportTemplate>("save_import_template", {
        name,
        columnMapping,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["import-templates"] });
    },
  });
}

export function useDeleteTemplate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      tauriInvoke<void>("delete_import_template", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["import-templates"] });
    },
  });
}

export function useExportAllData() {
  return useMutation({
    mutationFn: () => tauriInvoke<string>("export_all_data"),
  });
}

export function useImportBackup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (filePath: string) =>
      tauriInvoke<BackupImportResult>("import_backup", { filePath }),
    onSuccess: () => {
      queryClient.invalidateQueries();
    },
  });
}
