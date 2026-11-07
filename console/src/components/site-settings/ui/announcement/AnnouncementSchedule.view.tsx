import type { JSX } from "preact";
import { Field } from "../../../shared/ui/Field.view";
import { Switch } from "../../../shared/ui/Switch.view";
import { TextField } from "../../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";

export const AnnouncementSchedule = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <>
      <div class="grid gap-5 sm:grid-cols-2">
        <TextField
          label="Start"
          type="datetime-local"
          description={`Empty starts now. Time zone: ${data.announcement.timeZone}.`}
          value={data.form.announcement.starts_at}
          onValueChange={actions.announcement.text.starts_at}
          error={data.errors["announcement.starts_at"]}
        />
        <TextField
          label="End"
          type="datetime-local"
          description="Empty shows the bar with no end."
          value={data.form.announcement.ends_at}
          onValueChange={actions.announcement.text.ends_at}
          error={data.errors["announcement.ends_at"]}
        />
      </div>
      <Field label="Dismissible" description="Readers can close the bar.">
        <Switch
          checked={data.form.announcement.dismissible}
          onCheckedChange={actions.announcement.setDismissible}
        />
      </Field>
    </>
  );
};
