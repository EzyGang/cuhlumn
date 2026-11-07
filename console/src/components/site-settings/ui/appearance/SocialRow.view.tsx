import type { JSX } from "preact";
import { X } from "lucide-preact";
import { Button } from "../../../shared/ui/Button.view";
import { Select } from "../../../shared/ui/Select.view";
import { TextField } from "../../../shared/ui/TextField.view";
import type { Network } from "../../../../types/api/Network";
import type { SelectItem } from "../../../../types/select";
import type { SocialRowModel } from "../../../../types/siteSettings";

interface SocialRowProps {
  row: SocialRowModel;
  networks: readonly SelectItem<Network>[];
}

export const SocialRowView = ({ row, networks }: SocialRowProps): JSX.Element => (
  <div class="flex items-start gap-3">
    <Select
      label={row.networkText}
      value={row.network}
      items={networks}
      onValueChange={row.setNetwork}
      class="w-40 shrink-0"
    />
    <TextField
      label={row.urlText}
      value={row.url}
      onValueChange={row.setUrl}
      error={row.urlError}
      placeholder={row.placeholder}
      type="url"
      class="flex-1"
    />
    <div class="pt-[1.375rem]">
      <Button variant="ghost" label={row.removeText} onClick={row.remove}>
        <X size={14} aria-hidden="true" />
      </Button>
    </div>
  </div>
);
