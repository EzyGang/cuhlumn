import type { JSX } from "preact";
import type { ChipInputModel } from "../../../types/chips";
import { ChipList } from "./ChipList.view";
import { Field } from "./Field.view";
import { Input } from "./Input.view";

interface ChipInputProps {
  label: string;
  model: ChipInputModel;
  placeholder: string;
  error?: string | undefined;
  description?: string;
}

export const ChipInput = ({
  label,
  model,
  placeholder,
  error,
  description,
}: ChipInputProps): JSX.Element => (
  <Field label={label} error={error ?? model.error ?? undefined} description={description}>
    <div class="flex flex-col gap-2">
      <ChipList label={label} chips={model.chips} />
      <Input
        value={model.draft}
        onValueChange={model.setDraft}
        onKeyDown={model.keyDown}
        onBlur={model.commit}
        placeholder={model.full ? "The list is full" : placeholder}
        readOnly={model.full}
      />
    </div>
  </Field>
);
