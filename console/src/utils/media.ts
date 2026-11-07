/** The image types that the media upload accepts. */
export const MEDIA_ACCEPT = "image/png,image/jpeg,image/webp,image/gif,image/avif";

/** Returns the root-relative path of an uploaded image, which works behind any proxy or port. */
export const mediaPath = (name: string): string => `/media/${name}`;
