import type { JSX } from "preact";
import { Field } from "../../shared/ui/Field.view";
import { Switch } from "../../shared/ui/Switch.view";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";

export const IndexNowField = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="flex items-start gap-5">
      <TextField
        label="IndexNow key"
        description={
          data.mode === "create"
            ? "Leave empty to get a random key."
            : "8 to 128 letters, digits, or hyphens."
        }
        value={data.form.indexnow_key}
        onValueChange={actions.text.indexnow_key}
        error={data.errors["indexnow_key"]}
        mono
        class="flex-1"
      />
      <Field label="IndexNow" description="Pings on publish.">
        <Switch checked={data.form.indexnow} onCheckedChange={actions.toggle.indexnow} />
      </Field>
    </div>
  );
};
