import type { JSX } from "preact";
import { Save } from "lucide-preact";
import { Button } from "../../shared/ui/Button.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";

export const SiteSettingsHeader = (): JSX.Element => {
  const { data, status } = useSiteSettingsContext();
  return (
    <PageHeader
      eyebrow={data.mode === "create" ? "New site" : "Site settings"}
      title={data.form.title || data.form.host || "Untitled site"}
    >
      <Button type="submit" variant="primary" disabled={status.saving}>
        <Save size={14} aria-hidden="true" />
        {data.mode === "create" ? "Create site" : "Save changes"}
      </Button>
    </PageHeader>
  );
};
