const DATE_TIME = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" });

export const formatDateTime = (unixSeconds: number): string =>
  DATE_TIME.format(new Date(unixSeconds * 1000));

const COUNT = new Intl.NumberFormat();

export const formatCount = (count: number): string => COUNT.format(count);

export const formatBytes = (size: number): string => {
  if (size < 1024) {
    return `${size} B`;
  }
  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(1)} KB`;
  }
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
};
