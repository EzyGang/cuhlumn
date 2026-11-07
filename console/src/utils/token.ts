/** The shortest admin token that the server accepts, in characters. */
export const MIN_TOKEN_LENGTH = 32;

const TOKEN_BYTES = 32;

/** Returns a random admin token of 64 lowercase hex characters. */
export const generateToken = (): string => {
  const bytes = crypto.getRandomValues(new Uint8Array(TOKEN_BYTES));
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
};

/** Tells whether the server can accept the token as an admin token. */
export const isTokenLongEnough = (token: string): boolean => token.length >= MIN_TOKEN_LENGTH;
