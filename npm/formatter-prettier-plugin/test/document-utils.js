export const word = (value) => [
  value & 255,
  (value >>> 8) & 255,
  (value >>> 16) & 255,
  value >>> 24,
];
export const group = (id = 0, broken = false) => [
  3,
  Number(broken),
  ...word(id),
];
export const end = (tag) => [17, tag];
export const reference = (id) => [9, ...word(id)];
export const condition = (target, broken, flat) => [
  21,
  ...word(target),
  ...word(broken),
  ...word(flat),
];
export const alternatives = (...ids) => [
  10,
  ...word(ids.length),
  ...ids.flatMap(word),
];
export const completeGroup = (flags, id, contents, states) => [
  22,
  flags,
  ...word(id),
  ...word(contents),
  ...(states === undefined
    ? []
    : [...word(states.length), ...states.flatMap(word)]),
];
export const line = [2, 0];
export const hard = [2, 2];
export const rawHard = [2, 3];
export const literal = [2, 4];
export const rootLiteral = [2, 5];
export const rawLiteral = [2, 6];

export function wire(build, version = 0) {
  const strings = [];
  let length = 0;
  const text = (value) => {
    const bytes = Buffer.from(value);
    const op = [0, ...word(length), ...word(bytes.length)];
    strings.push(bytes);
    length += bytes.length;
    return op;
  };
  const fragments = build(text);
  return Buffer.concat([
    Buffer.from("SWCD"),
    Buffer.from([...word(version), ...word(fragments.length), ...word(length)]),
    ...fragments.flatMap((ops) => [
      Buffer.from(word(ops.length)),
      Buffer.from(ops),
    ]),
    ...strings,
  ]);
}
