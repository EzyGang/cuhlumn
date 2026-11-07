import { describe, expect, it } from "vitest";
import { writeClipboard } from "../utils/clipboard";

describe("writeClipboard", () => {
  it("writes the text when the clipboard API works", async () => {
    const written: string[] = [];
    const clipboard = {
      writeText: async (text: string): Promise<void> => {
        written.push(text);
      },
    };
    await expect(writeClipboard("harbor-token", clipboard)).resolves.toBe(true);
    expect(written).toEqual(["harbor-token"]);
  });

  it("reports a failure when the clipboard API is missing", async () => {
    await expect(writeClipboard("harbor-token", null)).resolves.toBe(false);
  });

  it("reports a failure when the browser refuses the text", async () => {
    const clipboard = {
      writeText: async (): Promise<void> => {
        throw new Error("not allowed");
      },
    };
    await expect(writeClipboard("harbor-token", clipboard)).resolves.toBe(false);
  });
});
