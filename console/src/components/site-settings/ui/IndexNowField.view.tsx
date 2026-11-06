import type { JSX } from "preact";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";

export const IndexNowField = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
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
    />
  );
};
