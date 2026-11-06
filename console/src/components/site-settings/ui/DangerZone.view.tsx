import type { JSX } from "preact";
import { Trash2 } from "lucide-preact";
import { ConfirmDialog } from "../../shared/ui/ConfirmDialog.view";
import { Section } from "../../shared/ui/Section.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";

export const DangerZone = (): JSX.Element => {
  const { data, status, actions } = useSiteSettingsContext();
  return (
    <Section title="Delete site" description="Deletes the site with all its posts and media.">
      <div>
        <ConfirmDialog
          trigger={
            <>
              <Trash2 size={14} aria-hidden="true" />
              Delete site
            </>
          }
          title={`Delete ${data.form.host}?`}
          description="All posts, pages, and media of this site are deleted. You cannot undo this."
          confirmLabel="Delete site"
          onConfirm={actions.remove}
          disabled={status.saving}
        />
      </div>
    </Section>
  );
};
