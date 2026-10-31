declare const A: {
  readonly state: number;
};
declare const B: {
  state: string;
};
declare const C: {
  get state(): number;
  set state(v: number);
};
declare const D: {
  get state(): number;
  set state(v: string);
};
declare const E: {};
declare const ObjectMethods: {
  a(): number;
  b(): number;
  c();
};
declare const ObjectKeys: {
  a: number;
  b: number;
  c: number;
  3: number;
  [-3]: number;
};
type family = "-apple-system, BlinkMacSystemFont, San Francisco, Segoe UI, Roboto, Helvetica Neue, sans-serif";
declare const family: family;
declare const theme: {
  readonly "font-family": family;
};
declare const X: {
  async: number;
  await: number;
};
declare const Y: {
  0: number;
  1: number;
  "2": number;
  3: number;
  4: number;
  "5": number;
};
declare const exoticKeys: {
  // --- WORKING KEYS ---
  a: string;
  é: string;
  ø: string;
  Ж: string;
  你: string;
  あ: string;
  한: string;
  á: string;
  // --- NON-WORKING KEYS ---
  "😃": string;
  "👍🏻": string;
  "👨‍👩‍👧‍👦": string;
  "±": string;
  "€": string;
  "“": string;
  "\0": string;
  "\\": string;
};
