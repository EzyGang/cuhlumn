import type { JSX } from "preact";
import { Images } from "lucide-preact";
import { Dialog } from "../../../shared/ui/Dialog.view";
import type { ImageKey, MediaPickerModel } from "../../../../types/siteSettings";
import { MediaChoicesView } from "./MediaChoices.view";
import { MediaUploadView } from "./MediaUpload.view";

interface MediaPickerDialogProps {
  label: string;
  name: ImageKey;
  picker: MediaPickerModel;
}

export const MediaPickerDialog = ({ label, name, picker }: MediaPickerDialogProps): JSX.Element => (
  <Dialog
    open={picker.target === name}
    onOpenChange={picker.setOpen[name]}
    trigger={
      <>
        <Images size={14} aria-hidden="true" />
        <span class="sr-only sm:not-sr-only">Choose image</span>
      </>
    }
    title={`Choose the ${label.toLowerCase()}`}
  >
    <div class="flex flex-col gap-5">
      <MediaUploadView picker={picker} />
      {picker.library ? (
        <MediaChoicesView items={picker.items} loading={picker.loading} error={picker.error} />
      ) : (
        <p class="text-sm text-secondary">The image uploads when you create the site.</p>
      )}
    </div>
  </Dialog>
);
