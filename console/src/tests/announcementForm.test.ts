import { describe, expect, it } from "vitest";
import {
  announcementState,
  announcementStatus,
  emptyAnnouncementForm,
  localInputToUnix,
  unixToLocalInput,
} from "../utils/announcementForm";

const local = (
  year: number,
  month: number,
  day: number,
  hours: number,
  minutes: number,
  seconds = 0,
): number => Math.floor(new Date(year, month, day, hours, minutes, seconds).getTime() / 1000);

describe("datetime-local conversion", () => {
  it("reads a local time without seconds", () => {
    expect(localInputToUnix("2026-11-09T09:30")).toBe(local(2026, 10, 9, 9, 30));
  });

  it("reads a local time with seconds", () => {
    expect(localInputToUnix("2026-11-09T09:30:15")).toBe(local(2026, 10, 9, 9, 30, 15));
  });

  it("returns null for a blank or broken value", () => {
    expect(localInputToUnix("")).toBeNull();
    expect(localInputToUnix("  ")).toBeNull();
    expect(localInputToUnix("2026-11-09")).toBeNull();
    expect(localInputToUnix("tomorrow")).toBeNull();
  });

  it("writes a minute without seconds and a time with seconds", () => {
    expect(unixToLocalInput(local(2026, 0, 2, 3, 4))).toBe("2026-01-02T03:04");
    expect(unixToLocalInput(local(2026, 0, 2, 3, 4, 5))).toBe("2026-01-02T03:04:05");
  });

  it("writes an empty value for a missing time", () => {
    expect(unixToLocalInput(null)).toBe("");
  });

  it("keeps unix seconds through a round trip", () => {
    for (const seconds of [1_700_000_000, 1_700_000_030, 1_798_761_600]) {
      expect(localInputToUnix(unixToLocalInput(seconds))).toBe(seconds);
    }
  });
});

describe("announcementStatus", () => {
  const now = 1_000;

  it("is live without bounds", () => {
    expect(announcementStatus({ starts_at: null, ends_at: null }, now)).toBe("live");
  });

  it("is scheduled before the start time and live at the start time", () => {
    expect(announcementStatus({ starts_at: now + 1, ends_at: null }, now)).toBe("scheduled");
    expect(announcementStatus({ starts_at: now, ends_at: null }, now)).toBe("live");
  });

  it("is ended at the end time and live just before it", () => {
    expect(announcementStatus({ starts_at: null, ends_at: now }, now)).toBe("ended");
    expect(announcementStatus({ starts_at: null, ends_at: now + 1 }, now)).toBe("live");
  });

  it("is live inside the window", () => {
    expect(announcementStatus({ starts_at: now - 10, ends_at: now + 10 }, now)).toBe("live");
  });
});

describe("announcementState", () => {
  const now = local(2026, 10, 9, 12, 0);

  it("is off when the bar is disabled, with any times", () => {
    const form = { ...emptyAnnouncementForm(), starts_at: "2026-11-10T09:00" };
    expect(announcementState(form, now)).toBe("off");
  });

  it("reads the form times in the browser time zone", () => {
    const form = { ...emptyAnnouncementForm(), enabled: true, text: "How we back up SQLite" };
    expect(announcementState({ ...form, starts_at: "2026-11-10T09:00" }, now)).toBe("scheduled");
    expect(announcementState({ ...form, ends_at: "2026-11-09T12:00" }, now)).toBe("ended");
    expect(announcementState(form, now)).toBe("live");
  });
});
