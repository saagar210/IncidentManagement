import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Upload, FileSpreadsheet, ArrowRight, ArrowLeft, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { ColumnMapper, autoDetectMappings } from "./column-mapper";
import { ImportPreviewTable } from "./import-preview";
import {
  useParseCSVHeaders,
  usePreviewImport,
  useExecuteImport,
  useImportTemplates,
  useSaveTemplate,
} from "@/hooks/use-import";
import type {
  ColumnMapping,
  ImportPreview,
  ImportResult,
} from "@/hooks/use-import";
import { toast } from "@/components/ui/use-toast";

type Step = "file" | "template" | "mapping" | "preview" | "result";

interface CSVImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CSVImportDialog({
  open: isOpen,
  onOpenChange,
}: CSVImportDialogProps) {
  const [step, setStep] = useState<Step>("file");
  const [filePath, setFilePath] = useState("");
  const [fileName, setFileName] = useState("");
  const [csvColumns, setCsvColumns] = useState<string[]>([]);
  const [mappings, setMappings] = useState<Record<string, string>>({});
  const [defaultValues, setDefaultValues] = useState<Record<string, string>>(
    {}
  );
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [templateName, setTemplateName] = useState("");

  const parseHeaders = useParseCSVHeaders();
  const previewImport = usePreviewImport();
  const executeImport = useExecuteImport();
  const { data: templates } = useImportTemplates();
  const saveTemplate = useSaveTemplate();

  const reset = useCallback(() => {
    setStep("file");
    setFilePath("");
    setFileName("");
    setCsvColumns([]);
    setMappings({});
    setDefaultValues({});
    setPreview(null);
    setResult(null);
    setTemplateName("");
  }, []);

  const handleOpenChange = useCallback(
    (open: boolean) => {
      if (!open) {
        reset();
      }
      onOpenChange(open);
    },
    [onOpenChange, reset]
  );

  // Step 1: Select CSV file
  const handleSelectFile = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "CSV Files", extensions: ["csv"] }],
    });
    if (selected) {
      const path = selected;
      setFilePath(path);
      setFileName(path.split("/").pop() ?? path.split("\\").pop() ?? path);

      try {
        const headers = await parseHeaders.mutateAsync(path);
        setCsvColumns(headers);
        // Auto-detect mappings
        const detected = autoDetectMappings(headers);
        setMappings(detected);

        // If templates exist, go to template step; otherwise skip to mapping
        if (templates && templates.length > 0) {
          setStep("template");
        } else {
          setStep("mapping");
        }
      } catch (err) {
        toast({
          title: "Failed to parse CSV",
          description: String(err),
          variant: "destructive",
        });
      }
    }
  };

  // Step 2: Apply a saved template
  const handleApplyTemplate = (templateId: string) => {
    const tpl = templates?.find((t) => t.id === templateId);
    if (tpl) {
      try {
        const parsed = JSON.parse(tpl.column_mapping);
        if (!parsed || typeof parsed.mappings !== "object" || typeof parsed.default_values !== "object") {
          throw new Error("Invalid template structure");
        }
        const validated = parsed as ColumnMapping;
        setMappings(validated.mappings);
        setDefaultValues(validated.default_values);
      } catch {
        toast({
          title: "Invalid template",
          description: "Could not parse template mapping",
          variant: "destructive",
        });
      }
    }
    setStep("mapping");
  };

  // Step 3: Handle mapping change
  const handleMappingChange = (csvColumn: string, incidentField: string) => {
    setMappings((prev) => {
      const next = { ...prev };
      if (incidentField === "") {
        delete next[csvColumn];
      } else {
        next[csvColumn] = incidentField;
      }
      return next;
    });
  };

  // Step 3 -> 4: Generate preview
  const handlePreview = async () => {
    const mapping: ColumnMapping = {
      mappings,
      default_values: defaultValues,
    };
    try {
      const p = await previewImport.mutateAsync({
        filePath,
        mapping,
      });
      setPreview(p);
      setStep("preview");
    } catch (err) {
      toast({
        title: "Preview failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  // Step 4 -> 5: Execute import
  const handleExecute = async () => {
    const mapping: ColumnMapping = {
      mappings,
      default_values: defaultValues,
    };
    try {
      const r = await executeImport.mutateAsync({
        filePath,
        mapping,
      });
      setResult(r);
      setStep("result");
    } catch (err) {
      toast({
        title: "Import failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  // Save current mapping as template
  const handleSaveTemplate = async () => {
    if (!templateName.trim()) return;
    const mapping: ColumnMapping = {
      mappings,
      default_values: defaultValues,
    };
    try {
      await saveTemplate.mutateAsync({
        name: templateName,
        columnMapping: JSON.stringify(mapping),
      });
      toast({ title: "Template saved", description: `"${templateName}" saved successfully` });
      setTemplateName("");
    } catch (err) {
      toast({
        title: "Save failed",
        description: String(err),
        variant: "destructive",
      });
    }
  };

  const mappedFieldCount = Object.values(mappings).filter(Boolean).length;

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileSpreadsheet className="h-5 w-5" />
            Import Incidents from CSV
          </DialogTitle>
          <DialogDescription>
            {step === "file" && "Select a CSV file to import incidents from."}
            {step === "template" && "Use a saved template or map columns manually."}
            {step === "mapping" && "Map CSV columns to incident fields."}
            {step === "preview" && "Review the import before confirming."}
            {step === "result" && "Import complete."}
          </DialogDescription>
        </DialogHeader>

        {/* Step 1: File Selection */}
        {step === "file" && (
          <div className="flex flex-col items-center gap-4 py-8">
            <Upload className="h-12 w-12 text-muted-foreground" />
            <p className="text-sm text-muted-foreground">
              Choose a .csv file to import
            </p>
            <Button onClick={handleSelectFile} disabled={parseHeaders.isPending}>
              {parseHeaders.isPending ? "Parsing..." : "Select CSV File"}
            </Button>
          </div>
        )}

        {/* Step 2: Template Selection */}
        {step === "template" && (
          <div className="space-y-4">
            <div className="rounded border p-3 flex items-center gap-2 bg-muted/50 text-sm">
              <FileSpreadsheet className="h-4 w-4 text-muted-foreground shrink-0" />
              <span className="truncate">{fileName}</span>
              <span className="text-muted-foreground">
                ({csvColumns.length} columns)
              </span>
            </div>

            {templates && templates.length > 0 && (
              <div className="space-y-2">
                <Label>Use saved template:</Label>
                <div className="space-y-1">
                  {templates.map((tpl) => (
                    <button
                      key={tpl.id}
                      onClick={() => handleApplyTemplate(tpl.id)}
                      className="w-full text-left rounded border px-3 py-2 text-sm hover:bg-accent transition-colors"
                    >
                      {tpl.name}
                    </button>
                  ))}
                </div>
              </div>
            )}

            <Button
              variant="outline"
              onClick={() => setStep("mapping")}
              className="w-full"
            >
              Map Columns Manually
            </Button>
          </div>
        )}

        {/* Step 3: Column Mapping */}
        {step === "mapping" && (
          <div className="space-y-4">
            <div className="rounded border p-3 flex items-center gap-2 bg-muted/50 text-sm">
              <FileSpreadsheet className="h-4 w-4 text-muted-foreground shrink-0" />
              <span className="truncate">{fileName}</span>
              <span className="text-muted-foreground">
                ({mappedFieldCount} fields mapped)
              </span>
            </div>

            <div className="max-h-[350px] overflow-y-auto pr-1">
              <ColumnMapper
                csvColumns={csvColumns}
                mappings={mappings}
                onMappingChange={handleMappingChange}
              />
            </div>

            {/* Save as template */}
            <div className="flex gap-2">
              <Input
                placeholder="Template name..."
                value={templateName}
                onChange={(e) => setTemplateName(e.target.value)}
                className="flex-1"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={handleSaveTemplate}
                disabled={!templateName.trim() || saveTemplate.isPending}
              >
                Save Template
              </Button>
            </div>

            <div className="flex justify-between">
              <Button
                variant="outline"
                onClick={() =>
                  setStep(templates && templates.length > 0 ? "template" : "file")
                }
              >
                <ArrowLeft className="h-4 w-4 mr-1" />
                Back
              </Button>
              <Button
                onClick={handlePreview}
                disabled={mappedFieldCount === 0 || previewImport.isPending}
              >
                {previewImport.isPending ? "Loading..." : "Preview"}
                <ArrowRight className="h-4 w-4 ml-1" />
              </Button>
            </div>
          </div>
        )}

        {/* Step 4: Preview */}
        {step === "preview" && preview && (
          <div className="space-y-4">
            <ImportPreviewTable preview={preview} />

            <div className="flex justify-between">
              <Button variant="outline" onClick={() => setStep("mapping")}>
                <ArrowLeft className="h-4 w-4 mr-1" />
                Back to Mapping
              </Button>
              <Button
                onClick={handleExecute}
                disabled={
                  preview.ready_count === 0 || executeImport.isPending
                }
              >
                {executeImport.isPending
                  ? "Importing..."
                  : `Import ${preview.ready_count + preview.warning_count} Incident${
                      preview.ready_count + preview.warning_count !== 1 ? "s" : ""
                    }`}
              </Button>
            </div>
          </div>
        )}

        {/* Step 5: Result */}
        {step === "result" && result && (
          <div className="space-y-4 py-4">
            <div className="flex flex-col items-center gap-3">
              <div className="rounded-full bg-green-500/10 p-3">
                <Check className="h-8 w-8 text-green-600" />
              </div>
              <p className="text-lg font-medium">Import Complete</p>
            </div>

            <div className="rounded border p-4 space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created:</span>
                <span className="font-medium text-green-600">
                  {result.created}
                </span>
              </div>
              {result.updated > 0 && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Updated:</span>
                  <span className="font-medium text-blue-600">
                    {result.updated}
                  </span>
                </div>
              )}
              {result.skipped > 0 && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Skipped:</span>
                  <span className="font-medium text-yellow-600">
                    {result.skipped}
                  </span>
                </div>
              )}
            </div>

            {result.errors.length > 0 && (
              <div className="space-y-1">
                <p className="text-sm font-medium text-destructive">Errors:</p>
                <ul className="text-xs text-muted-foreground space-y-0.5 max-h-[100px] overflow-y-auto">
                  {result.errors.map((err, idx) => (
                    <li key={idx}>{err}</li>
                  ))}
                </ul>
              </div>
            )}

            <Button className="w-full" onClick={() => handleOpenChange(false)}>
              Done
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
