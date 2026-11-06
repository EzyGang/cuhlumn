import type { JSX } from "preact";
import { TextField } from "../../../shared/ui/TextField.view";
import type { ImageFieldModel, MediaPickerModel } from "../../../../types/siteSettings";
import { MediaPickerDialog } from "./MediaPickerDialog.view";

interface ImageFieldProps {
  field: ImageFieldModel;
  picker: MediaPickerModel;
}

export const ImageField = ({ field, picker }: ImageFieldProps): JSX.Element => (
  <div class="flex items-end gap-3">
    {field.thumbnail === null ? (
      <div class="size-[42px] shrink-0 border border-dashed border-border" aria-hidden="true" />
    ) : (
      <img
        src={field.thumbnail}
        alt=""
        class="size-[42px] shrink-0 border border-border bg-card object-contain"
      />
    )}
    <TextField
      label={field.label}
      value={field.value}
      onValueChange={field.setValue}
      error={field.error}
      placeholder="https:// or /media/…"
      class="flex-1"
    />
    <MediaPickerDialog label={field.label} name={field.name} picker={picker} />
  </div>
);
