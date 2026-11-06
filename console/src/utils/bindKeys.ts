/** Returns one callback for each key, which passes the key and the new value to `update`. */
export const bindKeys = <K extends string, V>(
  keys: readonly K[],
  update: (key: K, value: V) => void,
): Record<K, (value: V) => void> =>
  Object.fromEntries(keys.map((key) => [key, (value: V): void => update(key, value)])) as Record<
    K,
    (value: V) => void
  >;
