import type { Signal } from "@preact/signals";
import type { BadgeTone } from "../../shared/ui/Badge.view";
import type { Tone } from "../../../types/api/Tone";
import type { RadioCardItem } from "../../../types/radioCards";
import type {
  AnnouncementActions,
  AnnouncementForm,
  AnnouncementModel,
  AnnouncementState,
  AnnouncementTextKey,
  SiteForm,
  SiteFormUpdate,
} from "../../../types/siteSettings";
import { ANNOUNCEMENT_TEXT_MAX, announcementState } from "../../../utils/announcementForm";
import { bindKeys } from "../../../utils/bindKeys";

const TONES: readonly RadioCardItem<Tone>[] = [
  { value: "info", label: "Info", description: "A quiet neutral bar." },
  { value: "accent", label: "Accent", description: "The accent colour of the site." },
  { value: "warning", label: "Warning", description: "An amber bar for urgent news." },
];

const TEXT_KEYS: readonly AnnouncementTextKey[] = [
  "text",
  "link",
  "link_label",
  "starts_at",
  "ends_at",
];

const STATES: Record<AnnouncementState, [string, BadgeTone]> = {
  off: ["Off", "neutral"],
  scheduled: ["Scheduled", "neutral"],
  live: ["Live", "success"],
  ended: ["Ended", "warning"],
};

/** Returns the status and the bound callbacks of the announcement form. */
export const useAnnouncement = (
  form: Signal<SiteForm>,
  update: SiteFormUpdate,
): { announcement: AnnouncementModel; actions: AnnouncementActions } => {
  const current = form.value.announcement;
  const change = (patch: Partial<AnnouncementForm>, key: string): void =>
    update({ announcement: { ...form.value.announcement, ...patch } }, `announcement.${key}`);
  const state = announcementState(current, Math.floor(Date.now() / 1000));
  const [stateText, stateTone] = STATES[state];
  return {
    announcement: {
      state,
      stateText,
      stateTone,
      textCount: `${[...current.text.trim()].length}/${ANNOUNCEMENT_TEXT_MAX} characters`,
      timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      tones: TONES,
    },
    actions: {
      setEnabled: (enabled: boolean): void =>
        update({ announcement: { ...form.value.announcement, enabled } }, "announcement"),
      text: bindKeys(TEXT_KEYS, (key, value: string): void => change({ [key]: value }, key)),
      setTone: (tone: Tone): void => change({ tone }, "tone"),
      setDismissible: (dismissible: boolean): void => change({ dismissible }, "dismissible"),
    },
  };
};
