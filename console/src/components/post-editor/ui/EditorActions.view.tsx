import type { JSX } from "preact";
import { Trash2 } from "lucide-preact";
import { Button } from "../../shared/ui/Button.view";
import { ConfirmDialog } from "../../shared/ui/ConfirmDialog.view";
import { usePostEditorContext } from "../context/PostEditorContext";

export const EditorActions = (): JSX.Element => {
  const { data, status, actions } = usePostEditorContext();
  return (
    <>
      {data.mode === "edit" ? (
        <ConfirmDialog
          trigger={<Trash2 size={14} aria-hidden="true" />}
          triggerLabel="Delete post"
          title={`Delete ${data.form.slug}?`}
          description="The post is removed from the site. You cannot undo this."
          confirmLabel="Delete post"
          onConfirm={actions.remove}
          disabled={status.saving}
        />
      ) : null}
      <Button onClick={actions.secondary} disabled={status.saving}>
        {data.secondaryLabel}
      </Button>
      <Button variant="primary" onClick={actions.primary} disabled={status.saving}>
        {data.primaryLabel}
      </Button>
    </>
  );
};
