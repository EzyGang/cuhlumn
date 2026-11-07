import type { JSX } from "preact";
import { RadioCards } from "../../../shared/ui/RadioCards.view";
import { TextField } from "../../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";

export const AnnouncementFields = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <>
      <TextField
        label="Text"
        description={data.announcement.textCount}
        value={data.form.announcement.text}
        onValueChange={actions.announcement.text.text}
        error={data.errors["announcement.text"]}
        placeholder="We moved our backups to a new region."
      />
      <div class="grid gap-5 sm:grid-cols-[2fr_1fr]">
        <TextField
          label="Link"
          type="url"
          value={data.form.announcement.link}
          onValueChange={actions.announcement.text.link}
          error={data.errors["announcement.link"]}
          placeholder="https:// or /path"
        />
        <TextField
          label="Link text"
          value={data.form.announcement.link_label}
          onValueChange={actions.announcement.text.link_label}
          error={data.errors["announcement.link_label"]}
          placeholder="Learn more"
        />
      </div>
      <RadioCards
        label="Tone"
        value={data.form.announcement.tone}
        items={data.announcement.tones}
        onValueChange={actions.announcement.setTone}
        columns={3}
      />
    </>
  );
};
