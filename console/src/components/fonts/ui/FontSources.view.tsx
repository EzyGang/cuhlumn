import type { JSX } from "preact";
import { Download, Upload } from "lucide-preact";
import { Button } from "../../shared/ui/Button.view";
import { Section } from "../../shared/ui/Section.view";
import { TextField } from "../../shared/ui/TextField.view";
import type { FontsModel } from "../../../types/fonts";

export const FontSources = ({ data, status, actions }: FontsModel): JSX.Element => (
  <div class="grid gap-4 lg:grid-cols-2">
    <Section title="Upload" description="TTF, OTF, or TTC files, up to 20 MB each.">
      <label class="inline-flex w-fit cursor-pointer items-center gap-2 border border-border px-4 py-3 text-xs font-medium tracking-wide uppercase transition-colors duration-150 focus-within:outline-2 focus-within:outline-offset-3 focus-within:outline-white hover:border-hover-border hover:bg-hover">
        <Upload size={14} aria-hidden="true" />
        {status.uploading ? "Uploading" : "Choose font files"}
        <input
          type="file"
          multiple
          accept={data.accept}
          class="sr-only"
          onChange={actions.pickFiles}
          disabled={status.uploading}
        />
      </label>
    </Section>
    <Section title="Google Fonts" description="Paste the address of a font page or a stylesheet.">
      <form class="flex flex-col gap-4" onSubmit={actions.importGoogle}>
        <TextField
          label="Google Fonts address"
          type="url"
          value={data.googleUrl}
          onValueChange={actions.setGoogleUrl}
          placeholder="https://fonts.google.com/specimen/Noto+Sans+JP"
          error={data.urlError}
        />
        <div>
          <Button type="submit" disabled={status.importing || data.googleUrl.trim() === ""}>
            <Download size={14} aria-hidden="true" />
            {status.importing ? "Importing" : "Import"}
          </Button>
        </div>
      </form>
    </Section>
  </div>
);
