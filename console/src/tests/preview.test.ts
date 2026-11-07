import { describe, expect, it } from "vitest";
import { previewDocument, toPreviewModel } from "../utils/preview";

const SITE_CSS = ":root{--color-bg:#f4ecd8}.post-body pre{background:#2b303b}";
const HTML = '<pre><code><span class="hl-keyword">fn</span> main() {}</code></pre>';

describe("previewDocument", () => {
  it("puts the site stylesheet in the head", () => {
    const document = previewDocument(HTML, SITE_CSS);
    expect(document).toContain(`<style>${SITE_CSS}</style></head>`);
  });

  it("wraps the HTML in the post page structure that the site rules select", () => {
    const document = previewDocument(HTML, SITE_CSS);
    expect(document).toContain(
      `<main class="column"><article><div class="post-body">${HTML}</div></article></main>`,
    );
  });

  it("keeps a closing style tag in the stylesheet from ending the style element", () => {
    const document = previewDocument("<p>Harbor Notes</p>", "a{}</style><script>x()</script>");
    expect(document.match(/<\/style>/g)).toHaveLength(1);
  });
});

describe("toPreviewModel", () => {
  it("builds the document from the HTML and the stylesheet of a ready preview", () => {
    const model = toPreviewModel({
      kind: "ready",
      html: HTML,
      css: SITE_CSS,
      toc: [],
      wordCount: 1,
    });
    expect(model.document).toBe(previewDocument(HTML, SITE_CSS));
    expect(model.wordCount).toBe("1 word");
  });
});
