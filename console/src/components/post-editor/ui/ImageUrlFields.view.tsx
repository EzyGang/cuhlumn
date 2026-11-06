import type { JSX } from "preact";
import { TextField } from "../../shared/ui/TextField.view";
import { usePostEditorContext } from "../context/PostEditorContext";

export const ImageUrlFields = (): JSX.Element => {
  const { data, actions } = usePostEditorContext();
  return (
    <div class="grid gap-5 md:grid-cols-2">
      <TextField
        label="Cover image"
        value={data.form.cover}
        onValueChange={actions.text.cover}
        error={data.errors["cover"]}
        placeholder="https:// or /media/…"
      />
      <TextField
        label="Share image"
        value={data.form.og_image}
        onValueChange={actions.text.og_image}
        error={data.errors["og_image"]}
        placeholder="https:// or /media/…"
      />
    </div>
  );
};
