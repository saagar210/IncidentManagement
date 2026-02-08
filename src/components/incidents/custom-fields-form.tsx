import { useEffect, useState, useRef, useCallback } from "react";
import { useCustomFields, useIncidentCustomFields, useSetIncidentCustomFields } from "@/hooks/use-custom-fields";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { CustomFieldValue } from "@/types/custom-fields";

interface CustomFieldsFormProps {
  incidentId: string | undefined;
  onSaveRef?: (saveFn: (() => Promise<void>) | null) => void;
}

export function CustomFieldsForm({ incidentId, onSaveRef }: CustomFieldsFormProps) {
  const { data: fields } = useCustomFields();
  const { data: existingValues } = useIncidentCustomFields(incidentId);
  const setValues = useSetIncidentCustomFields();
  const [localValues, setLocalValues] = useState<Record<string, string>>({});

  // Keep a ref to the latest local values so save() always reads the current state
  const localValuesRef = useRef(localValues);
  localValuesRef.current = localValues;

  const setValuesRef = useRef(setValues);
  setValuesRef.current = setValues;

  // Initialize local values from existing data
  useEffect(() => {
    if (existingValues) {
      const map: Record<string, string> = {};
      for (const v of existingValues) {
        map[v.field_id] = v.value;
      }
      setLocalValues(map);
    }
  }, [existingValues]);

  // Stable save function that reads from refs — never goes stale
  const save = useCallback(async () => {
    if (!incidentId) return;
    const values: CustomFieldValue[] = Object.entries(localValuesRef.current)
      .filter(([_, v]) => v.trim().length > 0)
      .map(([fieldId, value]) => ({
        incident_id: incidentId,
        field_id: fieldId,
        value,
      }));
    await setValuesRef.current.mutateAsync({ incidentId, values });
  }, [incidentId]);

  // Register save function with parent — only re-registers when incidentId changes
  useEffect(() => {
    onSaveRef?.(save);
    return () => onSaveRef?.(null);
  }, [save, onSaveRef]);

  if (!fields || fields.length === 0) return null;

  const handleChange = (fieldId: string, value: string) => {
    setLocalValues((prev) => ({ ...prev, [fieldId]: value }));
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Custom Fields</CardTitle>
      </CardHeader>
      <CardContent className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        {fields.map((field) => {
          const value = localValues[field.id] ?? "";
          return (
            <div key={field.id}>
              <Label>{field.name}</Label>
              {field.field_type === "select" ? (
                <Select
                  value={value}
                  onChange={(e) => handleChange(field.id, e.target.value)}
                >
                  <option value="">Select...</option>
                  {field.options
                    .split(",")
                    .map((opt) => opt.trim())
                    .filter(Boolean)
                    .map((opt) => (
                      <option key={opt} value={opt}>
                        {opt}
                      </option>
                    ))}
                </Select>
              ) : (
                <Input
                  type={field.field_type === "number" ? "number" : "text"}
                  value={value}
                  onChange={(e) => handleChange(field.id, e.target.value)}
                  placeholder={`Enter ${field.name.toLowerCase()}`}
                />
              )}
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
