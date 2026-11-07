import type { JSX } from "preact";
import { Badge } from "../../../shared/ui/Badge.view";
import { Field } from "../../../shared/ui/Field.view";
import { Section } from "../../../shared/ui/Section.view";
import { Switch } from "../../../shared/ui/Switch.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { AnnouncementFields } from "./AnnouncementFields.view";
import { AnnouncementSchedule } from "./AnnouncementSchedule.view";

export const AnnouncementSection = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <Section title="Announcement" description="A short message above every page of the blog.">
      <div class="flex items-start justify-between gap-5">
        <Field
          label="Show the bar"
          description="The bar shows only between the start and end times."
          error={data.errors["announcement"]}
        >
          <Switch
            checked={data.form.announcement.enabled}
            onCheckedChange={actions.announcement.setEnabled}
          />
        </Field>
        <div role="status" aria-label="Announcement status">
          <Badge tone={data.announcement.stateTone}>{data.announcement.stateText}</Badge>
        </div>
      </div>
      {data.form.announcement.enabled ? (
        <>
          <AnnouncementFields />
          <AnnouncementSchedule />
        </>
      ) : null}
    </Section>
  );
};
