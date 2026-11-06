import type { JSX } from "preact";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";

export const ProjectFields = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <>
      <TextField
        label="Project name"
        value={data.form.project_name}
        onValueChange={actions.text.project_name}
        error={data.errors["project_name"]}
      />
      <TextField
        label="Project URL"
        type="url"
        value={data.form.project_url}
        onValueChange={actions.text.project_url}
        error={data.errors["project_url"]}
        placeholder="https://"
      />
    </>
  );
};
