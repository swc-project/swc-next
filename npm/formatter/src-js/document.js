// SWCD v0. Keep the wire tags in sync with the Rust and Prettier decoders.

const utf8 = new TextEncoder();
const invalidDoc = () => new TypeError("Invalid document");

function word(output, value) {
  if (!Number.isInteger(value) || value < 0 || value > 0xffffffff) {
    throw new RangeError("SWCD field exceeds u32 capacity");
  }
  output.push(
    value & 255,
    (value >>> 8) & 255,
    (value >>> 16) & 255,
    value >>> 24,
  );
}

function checkAcyclic(fragments) {
  const states = new Uint8Array(fragments.length);
  const pending = [{ id: 0, next: 0 }];
  states[0] = 1;
  while (pending.length !== 0) {
    const frame = pending.at(-1);
    const children = fragments[frame.id].children;
    if (frame.next === children.length) {
      states[frame.id] = 2;
      pending.pop();
      continue;
    }
    const child = children[frame.next++];
    if (states[child] === 1) throw new TypeError("Cyclic document");
    if (states[child] === 0) {
      states[child] = 1;
      pending.push({ id: child, next: 0 });
    }
  }
}

/**
 * Encodes a Prettier Doc, keeping opaque label values in a separate array.
 * Nonnegative alignment widths must be u32 integers. Finite negative alignment
 * values dedent once; -Infinity restores the marked root.
 */
export function encodeDoc(doc) {
  const ids = new Map();
  const groups = new Map();
  const labels = [];
  const fragments = [];
  const strings = new Map();
  let stringLength = 0;

  const fragment = (value) => {
    if (ids.has(value)) return ids.get(value);
    const id = fragments.length;
    ids.set(value, id);
    fragments.push({ value, children: [], ops: [] });
    return id;
  };
  const group = (id) => {
    if (id === undefined) return 0;
    if (typeof id !== "symbol") throw invalidDoc();
    if (!groups.has(id)) groups.set(id, groups.size + 1);
    return groups.get(id);
  };
  const text = (ops, value) => {
    if (/[\uD800-\uDFFF]/u.test(value)) {
      throw new TypeError("Document text contains an isolated surrogate");
    }
    if (!strings.has(value)) {
      const bytes = utf8.encode(value);
      strings.set(value, { bytes, offset: stringLength });
      stringLength += bytes.length;
    }
    const { bytes, offset } = strings.get(value);
    word(ops, offset);
    word(ops, bytes.length);
  };

  fragment(doc);
  let instructionLength = 0;
  for (let index = 0; index < fragments.length; index++) {
    const { value, children, ops } = fragments[index];
    const ref = (child) => {
      const id = fragment(child);
      children.push(id);
      word(ops, id);
    };
    const append = (child) => {
      ops.push(9);
      ref(child);
    };
    const scope = (tag, contents, parameter) => {
      ops.push(tag);
      parameter?.();
      append(contents);
      ops.push(17, tag);
    };

    if (typeof value === "string") {
      if (value.length !== 0) {
        ops.push(0);
        text(ops, value);
      }
    } else if (Array.isArray(value)) {
      for (const child of value) append(child);
    } else if (value && typeof value === "object") {
      switch (value.type) {
        case "group": {
          const states = value.expandedStates;
          if (
            states !== undefined &&
            (!Array.isArray(states) || states.length === 0)
          ) {
            throw invalidDoc();
          }
          const expanded = value.break === true || value.break === "propagated";
          ops.push(22, Number(expanded) | (states === undefined ? 0 : 2));
          word(ops, group(value.id));
          ref(value.contents);
          if (states !== undefined) {
            word(ops, states.length);
            for (const state of states) ref(state);
          }
          break;
        }
        case "indent":
          scope(4, value.contents);
          break;
        case "align": {
          const alignment = value.n;
          if (typeof alignment === "string") {
            scope(23, value.contents, () => text(ops, alignment));
          } else if (alignment === -Infinity) {
            scope(26, value.contents);
          } else if (Number.isFinite(alignment) && alignment < 0) {
            scope(24, value.contents);
          } else if (alignment?.type === "root") {
            scope(25, value.contents);
          } else {
            scope(6, value.contents, () => word(ops, alignment));
          }
          break;
        }
        case "indent-if-break":
          scope(7, value.contents, () => {
            ops.push(Number(Boolean(value.negate)));
            word(ops, group(value.groupId));
          });
          break;
        case "if-break":
          ops.push(21);
          word(ops, group(value.groupId));
          ref(value.breakContents);
          ref(value.flatContents ?? "");
          break;
        case "fill":
          if (!Array.isArray(value.parts)) throw invalidDoc();
          ops.push(11);
          for (const part of value.parts) scope(12, part);
          ops.push(17, 11);
          break;
        case "line-suffix":
          scope(14, value.contents);
          break;
        case "line-suffix-boundary":
          ops.push(15);
          break;
        case "break-parent":
          ops.push(16);
          break;
        case "trim":
          ops.push(27);
          break;
        case "cursor":
          ops.push(28);
          break;
        case "label":
          scope(29, value.contents, () => {
            word(ops, labels.length);
            labels.push(value.label);
          });
          break;
        case "line": {
          const soft = Number(Boolean(value.soft));
          const ordinary = value.literal ? 7 + soft : soft;
          const hard = value.literal ? 6 : 3;
          ops.push(2, value.hard ? hard : ordinary);
          break;
        }
        default:
          throw invalidDoc();
      }
    } else {
      throw invalidDoc();
    }
    instructionLength += ops.length + 4;
  }

  checkAcyclic(fragments);
  const header = [83, 87, 67, 68, 0, 0, 0, 0];
  word(header, fragments.length);
  word(header, stringLength);
  const buffer = new Uint8Array(
    header.length + instructionLength + stringLength,
  );
  const view = new DataView(buffer.buffer);
  buffer.set(header);
  let offset = header.length;
  for (const { ops } of fragments) {
    view.setUint32(offset, ops.length, true);
    offset += 4;
    buffer.set(ops, offset);
    offset += ops.length;
  }
  for (const { bytes, offset: start } of strings.values())
    buffer.set(bytes, offset + start);
  return { buffer, labels };
}
