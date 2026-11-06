const MAX_SLUG_BYTES = 100;
const RESERVED: Record<string, true> = { api: true, og: true, media: true, tags: true };

const trimToLength = (slug: string): string => {
  if (slug.length <= MAX_SLUG_BYTES) {
    return slug;
  }
  const cut = slug.slice(0, MAX_SLUG_BYTES);
  const lastDash = cut.lastIndexOf("-");
  return (lastDash > 0 ? cut.slice(0, lastDash) : cut).replace(/-+$/, "");
};

export const suggestSlug = (title: string): string => {
  const slug = trimToLength(
    title
      .normalize("NFKD")
      .replaceAll(/[\u0300-\u036f]/g, "")
      .toLowerCase()
      .replaceAll(/[^a-z0-9]+/g, "-")
      .replaceAll(/^-+|-+$/g, ""),
  );
  return RESERVED[slug] === true ? `${slug}-page` : slug;
};
