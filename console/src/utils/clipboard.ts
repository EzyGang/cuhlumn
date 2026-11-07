/** The part of the clipboard API that the console uses. */
export type TextClipboard = Pick<Clipboard, "writeText">;

/**
 * Writes the text to the clipboard and tells whether it worked.
 * Returns false when the clipboard API is missing or refuses the text.
 * The API is missing on plain HTTP pages outside localhost.
 */
export const writeClipboard = async (
  text: string,
  clipboard: TextClipboard | null = (
    globalThis.navigator as { clipboard?: TextClipboard } | undefined
  )?.clipboard ?? null,
): Promise<boolean> => {
  if (clipboard === null) {
    return false;
  }
  try {
    await clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
};
