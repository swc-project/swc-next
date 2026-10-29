import { builders as b } from "prettier/doc";

const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const invalid = () => new Error("Invalid SWCFMT document buffer");
const contents = (parts) => (parts.length === 1 ? parts[0] : parts);

// Private ABI v0; keep in sync with document/encode.rs. Fragments are complete
// scopes, so their standard Docs can be shared without caller-dependent state.
export function decode(buffer, source, options = { tabWidth: 2 }) {
  const bytes = new Uint8Array(
    buffer.buffer,
    buffer.byteOffset,
    buffer.byteLength,
  );
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let offset = 0;
  const word = () => {
    if (offset + 4 > bytes.length) throw invalid();
    const value = view.getUint32(offset, true);
    offset += 4;
    return value;
  };
  const magic = word();
  if (word() !== 0) throw invalid();
  if (magic === 0x45435753) {
    const start = word();
    const end = word();
    const sourceBytes = new TextEncoder().encode(source);
    if (start > end || end > sourceBytes.length) throw invalid();
    const location = (byte) => {
      const prefix = utf8.decode(sourceBytes.subarray(0, byte));
      const lines = prefix.split(/\r\n|[\n\r\u2028\u2029]/u);
      return { line: lines.length, column: lines.at(-1).length + 1 };
    };
    const error = new SyntaxError(utf8.decode(bytes.subarray(offset)));
    error.loc = { start: location(start), end: location(end) };
    throw error;
  }
  if (magic !== 0x44435753) throw invalid();
  const count = word();
  const stringLength = word();
  if (count === 0 || count > (bytes.length - offset) / 4) throw invalid();
  const fragments = [];
  for (let index = 0; index < count; index++) {
    const end = word() + offset;
    if (end > bytes.length) throw invalid();
    const byte = () => {
      if (offset >= end) throw invalid();
      return bytes[offset++];
    };
    const integer = () => {
      if (offset + 4 > end) throw invalid();
      return word();
    };
    const reference = () => {
      const id = integer();
      if (id >= count) throw invalid();
      return id;
    };
    const ops = [];
    while (offset < end) {
      const op = byte();
      switch (op) {
        case 0:
        case 23:
          ops.push([op, integer(), integer()]);
          break;
        case 2:
        case 17:
          ops.push([op, byte()]);
          break;
        case 3:
        case 8:
          ops.push([op, byte(), integer()]);
          break;
        case 4:
        case 11:
        case 12:
        case 14:
        case 15:
        case 16:
        case 24:
        case 25:
        case 26:
          ops.push([op]);
          break;
        case 6:
        case 7:
        case 19:
          ops.push([op, integer()]);
          break;
        case 9:
          ops.push([op, reference()]);
          break;
        case 10: {
          const length = integer();
          if (length < 1 || length > (end - offset) / 4) throw invalid();
          ops.push([op, Array.from({ length }, reference)]);
          break;
        }
        case 21:
          ops.push([op, integer(), reference(), reference()]);
          break;
        case 22: {
          const flags = byte();
          if (flags & ~3) throw invalid();
          const id = integer();
          const contents = reference();
          let states;
          if (flags & 2) {
            const length = integer();
            if (length < 1 || length > (end - offset) / 4) throw invalid();
            states = Array.from({ length }, reference);
          }
          ops.push([op, flags, id, contents, states]);
          break;
        }
        default:
          throw invalid();
      }
    }
    fragments.push(ops);
  }
  if (offset + stringLength !== bytes.length) throw invalid();
  const poolStart = offset;
  const strings = new Map();
  const text = (start, length) => {
    if (start + length > stringLength) throw invalid();
    const key = `${start}:${length}`;
    if (!strings.has(key))
      strings.set(
        key,
        utf8.decode(
          bytes.subarray(poolStart + start, poolStart + start + length),
        ),
      );
    return strings.get(key);
  };
  const groups = new Map();
  const group = (id) => {
    if (id === 0) return undefined;
    if (!groups.has(id)) groups.set(id, Symbol());
    return groups.get(id);
  };
  const docs = [];
  for (const id of fragmentOrder(fragments))
    docs[id] = build(fragments[id], docs, text, group, options);
  return docs[0];
}

// Validate every fragment, including unreachable ones, without recursive walks.
function fragmentOrder(fragments) {
  const references = fragments.map((ops) =>
    ops.flatMap(([op, first, second, third, fourth]) => {
      if (op === 9) return [first];
      if (op === 10) return first;
      if (op === 21) return [second, third];
      if (op === 22) return [third, ...(fourth ?? [])];
      return [];
    }),
  );
  const state = new Uint8Array(fragments.length);
  const order = [];
  for (let id = 0; id < fragments.length; id++) {
    if (state[id] === 2) continue;
    const work = [{ id, index: 0 }];
    state[id] = 1;
    while (work.length) {
      const frame = work.at(-1);
      const children = references[frame.id];
      if (frame.index === children.length) {
        state[frame.id] = 2;
        order.push(frame.id);
        work.pop();
        continue;
      }
      const child = children[frame.index++];
      if (state[child] === 1) throw invalid();
      if (state[child] === 0) {
        state[child] = 1;
        work.push({ id: child, index: 0 });
      }
    }
  }
  return order;
}

function build(ops, docs, text, group, options) {
  const stack = [{ op: -1, parts: [] }];
  const append = (doc) => stack.at(-1).parts.push(doc);
  for (const [op, first, second, third, fourth] of ops) {
    switch (op) {
      case 0: {
        const value = text(first, second);
        append(value);
        break;
      }
      case 2: {
        const line = [
          b.line,
          b.softline,
          b.hardline,
          b.hardlineWithoutBreakParent,
          b.literalline,
          b.markAsRoot(b.literalline),
          b.literallineWithoutBreakParent,
        ][first];
        if (!line) throw invalid();
        append(line);
        break;
      }
      case 3:
      case 4:
      case 6:
      case 7:
      case 8:
      case 11:
      case 12:
      case 14:
      case 19:
      case 23:
      case 24:
      case 25:
      case 26:
        if ((op === 3 || op === 8) && first > 1) throw invalid();
        if (op === 7 && first === 0) throw invalid();
        if (op === 12 && stack.at(-1).op !== 11) throw invalid();
        stack.push({ op, first, second, parts: [] });
        break;
      case 9:
        append(docs[first]);
        break;
      case 10:
        append(b.conditionalGroup(first.map((id) => docs[id])));
        break;
      case 15:
        append(b.lineSuffixBoundary);
        break;
      case 16:
        append(b.breakParent);
        break;
      case 21:
        // Both branches remain visible to Prettier's breakParent propagation.
        append(b.ifBreak(docs[second], docs[third], { groupId: group(first) }));
        break;
      case 22:
        append(
          b.group(docs[third], {
            id: group(second),
            shouldBreak: Boolean(first & 1),
            expandedStates: fourth?.map((id) => docs[id]),
          }),
        );
        break;
      case 17: {
        if (stack.length === 1) throw invalid();
        const frame = stack.pop();
        if (frame.op !== first && !(first === 4 && [7, 19].includes(frame.op)))
          throw invalid();
        const doc = contents(frame.parts);
        switch (frame.op) {
          case 3:
            append(
              b.group(doc, {
                shouldBreak: Boolean(frame.first),
                id: group(frame.second),
              }),
            );
            break;
          case 4:
            append(b.indent(doc));
            break;
          case 6:
            append(b.align(frame.first, doc));
            break;
          case 7:
            append(b.indentIfBreak(doc, { groupId: group(frame.first) }));
            break;
          case 8:
            append(
              b.ifBreak(frame.first ? doc : "", frame.first ? "" : doc, {
                groupId: group(frame.second),
              }),
            );
            break;
          case 11:
            append(b.fill(frame.parts));
            break;
          case 12:
            append(doc);
            break;
          case 14:
            append(b.lineSuffix(doc));
            break;
          case 19:
            append(
              frame.first > 0 && options.tabWidth > 0
                ? b.addAlignmentToDoc(doc, frame.first, options.tabWidth)
                : b.dedentToRoot(b.align(frame.first, doc)),
            );
            break;
          case 23:
            append(b.align(text(frame.first, frame.second), doc));
            break;
          case 24:
            append(b.dedent(doc));
            break;
          case 25:
            append(b.markAsRoot(doc));
            break;
          case 26:
            append(b.dedentToRoot(doc));
            break;
          default:
            throw invalid();
        }
        break;
      }
      default:
        throw invalid();
    }
  }
  if (stack.length !== 1) throw invalid();
  return contents(stack[0].parts);
}
