import type { JSX } from "preact";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";

export const LocaleFields = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="grid gap-5 sm:grid-cols-2">
      <TextField
        label="Language"
        value={data.form.lang}
        onValueChange={actions.text.lang}
        error={data.errors["lang"]}
        placeholder="en"
      />
      <TextField
        label="Home page"
        description="Slug of a page to show at the root. Leave empty for the post list."
        value={data.form.home}
        onValueChange={actions.text.home}
        error={data.errors["home"]}
      />
    </div>
  );
};
