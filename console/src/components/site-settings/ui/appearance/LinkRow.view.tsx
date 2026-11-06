import type { JSX } from "preact";
import { X } from "lucide-preact";
import { Button } from "../../../shared/ui/Button.view";
import { TextField } from "../../../shared/ui/TextField.view";
import type { LinkRowModel } from "../../../../types/siteSettings";

interface LinkRowProps {
  link: LinkRowModel;
}

export const LinkRowView = ({ link }: LinkRowProps): JSX.Element => (
  <div class="flex items-start gap-3">
    <TextField
      label={link.labelText}
      value={link.label}
      onValueChange={link.setLabel}
      error={link.labelError}
      class="w-40 shrink-0"
    />
    <TextField
      label={link.urlText}
      value={link.url}
      onValueChange={link.setUrl}
      error={link.urlError}
      placeholder="https:// or /path"
      class="flex-1"
    />
    <div class="pt-[1.375rem]">
      <Button variant="ghost" label={link.removeText} onClick={link.remove}>
        <X size={14} aria-hidden="true" />
      </Button>
    </div>
  </div>
);
