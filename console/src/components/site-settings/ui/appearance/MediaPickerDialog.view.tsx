import type { JSX } from "preact";
import { Images } from "lucide-preact";
import { Dialog } from "../../../shared/ui/Dialog.view";
import type { ImageKey, MediaPickerModel } from "../../../../types/siteSettings";
import { MediaChoicesView } from "./MediaChoices.view";

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
        <span class="sr-only sm:not-sr-only">Pick from media</span>
      </>
    }
    triggerDisabled={!picker.available}
    title={`Choose the ${label.toLowerCase()}`}
  >
    <MediaChoicesView items={picker.items} loading={picker.loading} error={picker.error} />
  </Dialog>
);
