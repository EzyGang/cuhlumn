import type { JSX } from "preact";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";
import { ProjectFields } from "./ProjectFields.view";

export const AuthorFields = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="grid gap-5 sm:grid-cols-2">
      <TextField
        label="Author"
        value={data.form.author}
        onValueChange={actions.text.author}
        error={data.errors["author"]}
      />
      <TextField
        label="Author URL"
        type="url"
        value={data.form.author_url}
        onValueChange={actions.text.author_url}
        error={data.errors["author_url"]}
        placeholder="https://"
      />
      <ProjectFields />
    </div>
  );
};
