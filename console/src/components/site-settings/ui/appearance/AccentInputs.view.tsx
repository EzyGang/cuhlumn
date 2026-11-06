import type { JSX } from "preact";
import { Input } from "../../../shared/ui/Input.view";
import { TextField } from "../../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";

export const AccentInputs = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="flex items-end gap-3">
      <Input
        type="color"
        label="Accent color picker"
        value={data.colorValue}
        onValueChange={actions.text.accent}
        class="h-[42px] w-14 shrink-0 cursor-pointer p-1"
      />
      <TextField
        label="Accent"
        value={data.form.accent}
        onValueChange={actions.text.accent}
        error={data.errors["brand.accent"]}
        placeholder="#2563eb"
        mono
        class="flex-1"
      />
    </div>
  );
};
