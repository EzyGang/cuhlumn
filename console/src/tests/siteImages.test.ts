import { describe, expect, it } from "vitest";
import type { PendingImage, PendingImages } from "../types/siteSettings";
import { emptySiteForm } from "../utils/siteForm";
import {
  changePending,
  imageUploads,
  uploadPending,
  withUploadedImages,
} from "../utils/siteImages";

const png = (name: string): File =>
  new File([new Uint8Array([137, 80, 78, 71])], name, {
    type: "image/png",
  });

const image = (name: string): PendingImage => ({
  file: png(name),
  preview: `data:image/png;base64,${name}`,
});

describe("pending image changes", () => {
  it("stores the chosen file for the field without a preview", () => {
    const file = png("harbor-logo.png");
    const pending = changePending({}, { type: "choose", key: "logo", file });
    expect(pending).toEqual({ logo: { file, preview: null } });
  });

  it("adds the preview when the field still holds the file", () => {
    const file = png("harbor-logo.png");
    const chosen = changePending({}, { type: "choose", key: "logo", file });
    const pending = changePending(chosen, {
      type: "preview",
      key: "logo",
      file,
      preview: "data:image/png;base64,AAAA",
    });
    expect(pending).toEqual({ logo: { file, preview: "data:image/png;base64,AAAA" } });
  });

  it("replaces the old file of the field and keeps the other fields", () => {
    const favicon = image("favicon.png");
    const next = png("harbor-logo-dark.png");
    const pending = changePending(
      { logo: image("harbor-logo.png"), favicon },
      { type: "choose", key: "logo", file: next },
    );
    expect(pending).toEqual({ logo: { file: next, preview: null }, favicon });
  });

  it("ignores the late preview of a file that the admin replaced", () => {
    const old = png("harbor-logo.png");
    const next = png("harbor-logo-dark.png");
    const replaced = changePending({}, { type: "choose", key: "logo", file: next });
    const pending = changePending(replaced, {
      type: "preview",
      key: "logo",
      file: old,
      preview: "data:image/png;base64,OLD",
    });
    expect(pending).toEqual({ logo: { file: next, preview: null } });
  });

  it("removes the file of the field and keeps the other fields", () => {
    const favicon = image("favicon.png");
    const pending = changePending(
      { logo: image("harbor-logo.png"), favicon },
      { type: "remove", key: "logo" },
    );
    expect(pending).toEqual({ favicon });
  });

  it("removes the file when the admin types an address in the field", () => {
    const pending = changePending(
      { og_image: image("share.png") },
      { type: "edit", key: "og_image" },
    );
    expect(pending).toEqual({});
  });

  it("ignores the preview of a file that the admin removed", () => {
    const file = png("favicon.png");
    const pending = changePending({}, { type: "preview", key: "favicon", file, preview: "data:," });
    expect(pending).toEqual({});
  });
});

describe("uploads after the create", () => {
  const pending: PendingImages = {
    og_image: image("share.png"),
    logo: image("harbor-logo.png"),
    favicon: image("favicon.png"),
  };

  it("gives each uploaded image its media path", async () => {
    const uploaded: string[] = [];
    const outcomes = await uploadPending(pending, async (file) => {
      uploaded.push(file.name);
      return { name: `0123abcd-${file.name}` };
    });
    expect(uploaded).toEqual(["harbor-logo.png", "favicon.png", "share.png"]);
    expect(imageUploads(outcomes)).toEqual({
      paths: {
        logo: "/media/0123abcd-harbor-logo.png",
        favicon: "/media/0123abcd-favicon.png",
        og_image: "/media/0123abcd-share.png",
      },
      failed: [],
    });
  });

  it("skips the field of a failed upload and uploads the other images", async () => {
    const failure = new Error("the file is too large");
    const outcomes = await uploadPending(pending, async (file) => {
      if (file.name === "favicon.png") {
        throw failure;
      }
      return { name: file.name };
    });
    const uploads = imageUploads(outcomes);
    expect(uploads.paths).toEqual({
      logo: "/media/harbor-logo.png",
      og_image: "/media/share.png",
    });
    expect(uploads.failed).toEqual([
      { key: "favicon", file: pending.favicon?.file, error: failure },
    ]);
  });

  it("uploads nothing when no file waits", async () => {
    const outcomes = await uploadPending({}, async () => ({ name: "unused.png" }));
    expect(imageUploads(outcomes)).toEqual({ paths: {}, failed: [] });
  });

  it("adds the paths to a form that changed during the save, except fields the admin changed", () => {
    const sent = { ...emptySiteForm(), host: "harbor.example.com", title: "Harbor Notes" };
    const typed = {
      ...sent,
      title: "Harbor Notes and News",
      favicon: "https://cdn.example.com/f.png",
    };
    const paths = { logo: "/media/logo.png", favicon: "/media/favicon.png" };
    const draft = withUploadedImages(typed, sent, paths);
    expect(draft.title).toBe("Harbor Notes and News");
    expect(draft.logo).toBe("/media/logo.png");
    expect(draft.favicon).toBe("https://cdn.example.com/f.png");
    expect(draft.og_image).toBe("");
  });
});
