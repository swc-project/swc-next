declare const A: {
  readonly state: number;
};
declare const B: {
  state: string;
} // do something
;
declare const C: {
  get state(): number;
  set state(v: number);
} // do something
;
declare const D: {
  get state(): number;
  set state(v: string);
} // do something
;
declare const E: {} // do something
;
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
  a: string;
  é: string;
  ø: string;
  Ж: string;
  你: string;
  あ: string;
  한: string;
  á: string;
  "😃": string;
  "👍🏻": string;
  "👨‍👩‍👧‍👦": string;
  "±": string;
  "€": string;
  "“": string;
  "\0": string;
  "\\": string;
} // --- WORKING KEYS ---
// latin
// latin-1 supplement
// extended latin
// cyrillic
// chinese
// japanese
// korean
// zalgo (a + combining acute accent)
// --- NON-WORKING KEYS ---
// emoji
// emoji + skin tone
// emoji sequence (family)
// math symbol
// currency
// curly quote
// null byte (not expected to work)
// backslash (unique error)
;
