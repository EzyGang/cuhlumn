import type { Announcement } from "../types/api/Announcement";
import type { AnnouncementStatus } from "../types/api/AnnouncementStatus";
import type { AnnouncementForm, AnnouncementState } from "../types/siteSettings";

export const ANNOUNCEMENT_TEXT_MAX = 160;

const pad = (value: number, width = 2): string => String(value).padStart(width, "0");

/** Matches the `datetime-local` value format, with optional seconds and fractions. */
const LOCAL_TIME = /^(\d{4,})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?(?:\.\d+)?$/;

/** Formats unix seconds as a `datetime-local` value in the browser time zone. */
export const unixToLocalInput = (seconds: number | null): string => {
  if (seconds === null) {
    return "";
  }
  const time = new Date(seconds * 1000);
  const date = `${pad(time.getFullYear(), 4)}-${pad(time.getMonth() + 1)}-${pad(time.getDate())}`;
  const clock = `${pad(time.getHours())}:${pad(time.getMinutes())}`;
  const rest = time.getSeconds() === 0 ? "" : `:${pad(time.getSeconds())}`;
  return `${date}T${clock}${rest}`;
};

/** Reads a `datetime-local` value in the browser time zone as unix seconds, or null when blank. */
export const localInputToUnix = (value: string): number | null => {
  const match = LOCAL_TIME.exec(value.trim());
  if (match === null) {
    return null;
  }
  const [year, month, day, hours, minutes, seconds] = match.slice(1).map((part) => Number(part));
  const time = new Date(year ?? 0, (month ?? 1) - 1, day, hours, minutes, seconds || 0);
  const millis = time.getTime();
  return Number.isNaN(millis) ? null : Math.floor(millis / 1000);
};

/** Returns the status of an announcement at `now`, with the rules of the server. */
export const announcementStatus = (
  bounds: Pick<Announcement, "starts_at" | "ends_at">,
  now: number,
): AnnouncementStatus => {
  if (bounds.starts_at !== null && now < bounds.starts_at) {
    return "scheduled";
  }
  if (bounds.ends_at !== null && now >= bounds.ends_at) {
    return "ended";
  }
  return "live";
};

/** Returns "off" for a disabled announcement, and its status otherwise. */
export const announcementState = (form: AnnouncementForm, now: number): AnnouncementState =>
  form.enabled
    ? announcementStatus(
        { starts_at: localInputToUnix(form.starts_at), ends_at: localInputToUnix(form.ends_at) },
        now,
      )
    : "off";

export const emptyAnnouncementForm = (): AnnouncementForm => ({
  enabled: false,
  text: "",
  link: "",
  link_label: "",
  tone: "info",
  starts_at: "",
  ends_at: "",
  dismissible: true,
});

export const announcementToForm = (announcement: Announcement | null): AnnouncementForm =>
  announcement === null
    ? emptyAnnouncementForm()
    : {
        enabled: true,
        text: announcement.text,
        link: announcement.link ?? "",
        link_label: announcement.link_label,
        tone: announcement.tone,
        starts_at: unixToLocalInput(announcement.starts_at),
        ends_at: unixToLocalInput(announcement.ends_at),
        dismissible: announcement.dismissible,
      };

export const formToAnnouncement = (form: AnnouncementForm): Announcement | null => {
  if (!form.enabled) {
    return null;
  }
  const link = form.link.trim();
  return {
    text: form.text,
    link: link === "" ? null : link,
    link_label: form.link_label.trim(),
    tone: form.tone,
    starts_at: localInputToUnix(form.starts_at),
    ends_at: localInputToUnix(form.ends_at),
    dismissible: form.dismissible,
  };
};
