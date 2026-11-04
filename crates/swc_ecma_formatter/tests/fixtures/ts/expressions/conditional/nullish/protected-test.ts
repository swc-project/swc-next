declare const invalid: string;
const result = (
  // @ts-expect-error
  invalid() ?? 0 ? 0 : 0
);
