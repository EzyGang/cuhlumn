import type { JSX } from "preact";
import { Upload } from "lucide-preact";
import type { MediaPickerModel } from "../../../../types/siteSettings";

interface MediaUploadProps {
  picker: MediaPickerModel;
}

export const MediaUploadView = ({ picker }: MediaUploadProps): JSX.Element => (
  <div class="flex items-center gap-3">
    <label
      data-disabled={picker.uploading}
      class="inline-flex cursor-pointer items-center gap-2 border border-border px-4 py-3 text-xs font-medium tracking-wide uppercase transition-colors duration-150 focus-within:outline-2 focus-within:outline-offset-3 focus-within:outline-white hover:border-hover-border hover:bg-hover data-[disabled=true]:cursor-wait data-[disabled=true]:opacity-60"
    >
      <Upload size={14} aria-hidden="true" />
      Upload an image
      <input
        type="file"
        accept={picker.accept}
        class="sr-only"
        onChange={picker.upload}
        disabled={picker.uploading}
      />
    </label>
    {picker.uploading ? (
      <p class="text-sm text-muted" role="status">
        Uploading
      </p>
    ) : null}
  </div>
);
