import type { JSX } from "preact";
import { Select } from "../../../shared/ui/Select.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { AccentInputs } from "./AccentInputs.view";

export const AccentFields = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="grid gap-5 sm:grid-cols-2">
      <AccentInputs />
      <Select
        label="Font"
        value={data.form.font}
        items={data.fonts}
        onValueChange={actions.setFont}
      />
    </div>
  );
};
