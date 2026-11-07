import type { ReadonlySignal, Signal } from "@preact/signals";
import { effect, useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { ApiError } from "../../../services/http";
import { previewMarkdown } from "../../../services/posts.service";
import type { PostForm, PreviewModel, PreviewState } from "../../../types/postEditor";
import { toPreviewModel } from "../../../utils/preview";
import { errorMessage, isUnauthorized, reportApiError } from "../../../utils/reportApiError";

const DEBOUNCE_MS = 300;

const renderPreview = async (host: string, body: string): Promise<PreviewState> => {
  try {
    const preview = await previewMarkdown(host, { body });
    return {
      kind: "ready",
      html: preview.html,
      css: preview.css,
      toc: preview.toc,
      wordCount: preview.word_count,
    };
  } catch (error) {
    if (isUnauthorized(error)) {
      reportApiError(error);
    }
    if (error instanceof ApiError && (error.status === 404 || error.status === 405)) {
      return { kind: "unavailable" };
    }
    return { kind: "failed", message: errorMessage(error) };
  }
};

/**
 * Watches the body in the form signal and renders it on the server 300 ms after the last change.
 * The watch reads the signal directly, so every change counts: typing, paste, and programmatic input.
 */
const watchBody = (
  host: string,
  form: ReadonlySignal<PostForm>,
  state: Signal<PreviewState>,
): (() => void) => {
  let timer: number | undefined;
  let request = 0;
  let last: string | null = null;
  const stop = effect((): void => {
    const body = form.value.body;
    if (body === last) {
      return;
    }
    last = body;
    window.clearTimeout(timer);
    request += 1;
    const id = request;
    if (body.trim() === "") {
      state.value = { kind: "empty" };
      return;
    }
    timer = window.setTimeout((): void => {
      if (state.peek().kind !== "ready") {
        state.value = { kind: "loading" };
      }
      void renderPreview(host, body).then((next): void => {
        if (id === request) {
          state.value = next;
        }
      });
    }, DEBOUNCE_MS);
  });
  return (): void => {
    stop();
    window.clearTimeout(timer);
    request += 1;
  };
};

export const usePreview = (host: string, form: ReadonlySignal<PostForm>): PreviewModel => {
  const state = useSignal<PreviewState>({ kind: "empty" });
  useEffect((): (() => void) => watchBody(host, form, state), [host, form, state]);
  return toPreviewModel(state.value);
};
