import { builders as b } from "prettier/doc";

const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const invalid = () => new Error("Invalid SWCFMT document buffer");

// Private ABI; keep in sync with document/encode.rs. Never expose native pointers.
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
    // SWCE
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
  if (magic !== 0x44435753) throw invalid(); // SWCD
  const count = word();
  const stringLength = word();
  if (count === 0 || count > (bytes.length - offset) / 4) throw invalid();
  const fragments = [];
  for (let i = 0; i < count; i++) {
    const length = word();
    const end = offset + length;
    if (end > bytes.length) throw invalid();
    const ops = [];
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
    while (offset < end) {
      const op = byte();
      switch (op) {
        case 0:
          ops.push([op, integer(), integer()]);
          break;
        case 1:
        case 4:
        case 11:
        case 12:
        case 13:
        case 14:
        case 15:
        case 16:
          ops.push([op]);
          break;
        case 2:
        case 5:
        case 17:
          ops.push([op, byte()]);
          break;
        case 3:
        case 8:
          ops.push([op, byte(), integer()]);
          break;
        case 6:
        case 7:
        case 18:
        case 19:
          ops.push([op, integer()]);
          break;
        case 9:
          ops.push([op, reference()]);
          break;
        case 10:
        case 20: {
          const length = integer();
          if (length < 1 || length > (end - offset) / 4) throw invalid();
          ops.push([op, Array.from({ length }, reference)]);
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
  const canonical = { sourceGroups: new WeakSet(), cache: new WeakMap() };
  const groups = new Map();
  const groupId = (id) => {
    if (id === 0) return undefined;
    if (!groups.has(id)) groups.set(id, Symbol());
    return groups.get(id);
  };
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
  // Fill entries and alternatives can change the print mode without opening a
  // logical group. Shared scopes must preserve which mode their conditions use.
  const cache = Array.from({ length: count }, () => new Map());
  const active = new Uint8Array(count);
  const statesFor = (id, parentGroup, rawSpace, modeMatchesGroup) => {
    if (!cache[id].has(parentGroup))
      cache[id].set(parentGroup, [new Map(), new Map(), new Map(), new Map()]);
    return cache[id].get(parentGroup)[
      Number(rawSpace) | (Number(modeMatchesGroup) << 1)
    ];
  };
  const createFrame = (
    id,
    pendingSpace,
    parentGroup,
    rawSpace,
    modeMatchesGroup,
  ) => ({
    id,
    cache: statesFor(id, parentGroup, rawSpace, modeMatchesGroup),
    key: pendingSpace,
    iterator: build(
      fragments[id],
      text,
      groupId,
      pendingSpace,
      parentGroup,
      rawSpace,
      modeMatchesGroup,
      options,
      canonical,
    ),
    input: undefined,
  });
  const work = [createFrame(0, false, undefined, false, true)];
  active[0] = 1;
  while (work.length) {
    const frame = work.at(-1);
    const step = frame.iterator.next(frame.input);
    frame.input = undefined;
    if (step.done) {
      frame.cache.set(frame.key, step.value);
      active[frame.id] = 0;
      work.pop();
      if (work.length === 0) return step.value.doc;
      work.at(-1).input = step.value;
      continue;
    }
    const { id, pendingSpace, parentGroup, rawSpace, modeMatchesGroup } =
      step.value;
    if (active[id]) throw invalid();
    const cached = statesFor(id, parentGroup, rawSpace, modeMatchesGroup).get(
      pendingSpace,
    );
    if (cached) {
      frame.input = cached;
    } else {
      active[id] = 1;
      work.push(
        createFrame(id, pendingSpace, parentGroup, rawSpace, modeMatchesGroup),
      );
    }
  }
}

const contents = (parts) => (parts.length === 1 ? parts[0] : parts);

function* build(
  ops,
  text,
  groupId,
  pendingSpace,
  parentGroup,
  rawSpace,
  modeMatchesGroup,
  options,
  canonical,
) {
  const stack = [{ op: -1, parts: [] }];
  const append = (doc) => stack.at(-1).parts.push(doc);
  // Defer discardable spaces until content claims them. Carry this
  // state through references just as the native printer expands their elements.
  const flushSpace = () => {
    if (pendingSpace) append(pendingSpace === true ? " " : pendingSpace);
    pendingSpace = false;
  };
  for (const [op, first, second] of ops) {
    switch (op) {
      case 0:
        flushSpace();
        // Raw text (templates / ignored source / block comments) needs literal
        // line Docs so Prettier can stop fitting and reset its column there.
        // Group expansion is already encoded by the Rust document builder.
        {
          const source = text(first, second);
          const lines = source.split(/\r\n|[\n\r]/u);
          for (let index = 0; index < lines.length; index++) {
            if (index !== 0) append(b.literallineWithoutBreakParent);
            if (lines[index]) append(lines[index]);
          }
          if (source) rawSpace = /[ \t]$/u.test(source);
        }
        break;
      case 1:
        pendingSpace = true;
        break;
      case 2: {
        // Native line breaks retain whitespace that belongs to raw text,
        // including Markdown hard breaks in comments. Root alignment keeps the
        // active indentation when a literal line preserves that whitespace.
        const literal = rawSpace
          ? b.markAsRoot(b.literallineWithoutBreakParent)
          : b.hardlineWithoutBreakParent;
        const line = [
          rawSpace ? b.ifBreak(literal, " ") : b.line,
          rawSpace ? b.ifBreak(literal, "") : b.softline,
          rawSpace ? literal : b.hardline,
          literal,
          b.literallineWithoutBreakParent,
        ][first];
        if (!line) throw invalid();
        if (
          first === 1 &&
          modeMatchesGroup &&
          pendingSpace &&
          pendingSpace !== true
        ) {
          // A conditional gap can belong to the group's flat branch. Keep it
          // after the soft line so a broken probe discards it before measuring.
          // Unconditional gaps also encode reserved spaces, whose fit width
          // must remain before the line until the transport distinguishes them.
          append(line);
          pendingSpace =
            pendingSpace && parentGroup !== undefined
              ? b.ifBreak("", pendingSpace === true ? " " : pendingSpace, {
                  groupId: parentGroup,
                })
              : false;
          break;
        }
        if (first === 1) {
          flushSpace();
        } else {
          // Line supplies the flat gap itself, and discards it when broken.
          // Spaces adjacent to that line must not create a second gap.
          pendingSpace = false;
          rawSpace = false;
        }
        append(line);
        break;
      }
      case 3:
      case 4:
      case 5:
      case 6:
      case 7:
      case 11:
      case 12:
      case 13:
      case 19:
        // Native gaps remain pending across structural scopes.
        {
          const frame = {
            op,
            first,
            second,
            parts: [],
            parentGroup,
            modeMatchesGroup,
          };
          if (op === 12 || op === 13) modeMatchesGroup = false;
          if (op === 3) {
            parentGroup = frame.group = groupId(second) ?? Symbol();
            modeMatchesGroup = true;
          }
          stack.push(frame);
        }
        break;
      case 8:
        stack.push({
          op,
          first,
          second,
          parts: [],
          pendingSpace,
          rawSpace,
          group: second === 0xffffffff ? parentGroup : groupId(second),
          useCurrentMode:
            second === 0xffffffff &&
            modeMatchesGroup &&
            parentGroup !== undefined,
        });
        break;
      case 14:
        stack.push({ op, parts: [], pendingSpace, rawSpace });
        pendingSpace = false;
        rawSpace = false;
        break;
      case 9: {
        const result = yield {
          id: first,
          pendingSpace,
          parentGroup,
          rawSpace,
          modeMatchesGroup,
        };
        append(result.doc);
        pendingSpace = result.pendingSpace;
        rawSpace = result.rawSpace;
        break;
      }
      case 10:
      case 20: {
        const variants = [];
        const gaps = [];
        let variantRawSpace = false;
        let firstProjection;
        for (const [index, id] of first.entries()) {
          const result = yield {
            id,
            pendingSpace,
            parentGroup,
            rawSpace,
            modeMatchesGroup: false,
          };
          const projection =
            op === 20 && index < Math.max(1, first.length - 1)
              ? canonicalView(result.doc, canonical)
              : undefined;
          if (index === 0) firstProjection = projection;
          if (op === 20 && index < first.length - 1) {
            // Fitting sees the flat projection before this local group has a
            // mode. Printing commits Break first, then exposes the original
            // candidate with its source-only expansion intact.
            const selection = Symbol();
            variants.push(
              b.group(
                b.ifBreak(b.group(result.doc), projection, {
                  groupId: selection,
                }),
                { id: selection, shouldBreak: true },
              ),
            );
          } else {
            variants.push(result.doc);
          }
          gaps.push(result.pendingSpace);
          variantRawSpace ||= result.rawSpace;
        }
        rawSpace = variantRawSpace;
        const id = Symbol();
        const alternatives = b.conditionalGroup(variants, { id });
        if (op === 20) canonical.cache.set(alternatives, firstProjection);
        append(alternatives);
        pendingSpace = gaps.every((gap) => gap === gaps[0])
          ? gaps[0]
          : b.ifBreak(
              gaps.at(-1) === true ? " " : gaps.at(-1) || "",
              gaps[0] === true ? " " : gaps[0] || "",
              { groupId: id },
            );
        break;
      }
      case 15:
        append(b.lineSuffixBoundary);
        break;
      case 16:
        append(b.breakParent);
        break;
      case 17: {
        if (stack.length === 1) throw invalid();
        if (first === 14) flushSpace();
        const frame = stack.pop();
        if (frame.op !== first && !(first === 4 && [7, 19].includes(frame.op)))
          throw invalid();
        const doc = contents(frame.parts);
        switch (frame.op) {
          case 3:
            if (frame.first > 2) throw invalid();
            {
              const group = b.group(doc, {
                shouldBreak: Boolean(frame.first),
                id: frame.group,
              });
              if (frame.first === 2) canonical.sourceGroups.add(group);
              append(group);
            }
            parentGroup = frame.parentGroup;
            modeMatchesGroup = frame.modeMatchesGroup;
            break;
          case 4:
            append(b.indent(doc));
            break;
          case 5:
            if (frame.first > 1) throw invalid();
            append(frame.first ? b.dedentToRoot(doc) : b.dedent(doc));
            break;
          case 6:
            append(b.align(frame.first, doc));
            break;
          case 7:
            append(b.indentIfBreak(doc, { groupId: groupId(frame.first) }));
            break;
          case 19:
            append(
              b.dedentToRoot(
                options.tabWidth > 0
                  ? b.addAlignmentToDoc(doc, frame.first, options.tabWidth)
                  : b.align(frame.first, doc),
              ),
            );
            break;
          case 8:
            if (frame.first > 1) throw invalid();
            {
              const choose = (active, inactive, group) => {
                if (active === inactive) return active;
                if (frame.second === 0xffffffff && frame.group === undefined)
                  return frame.first ? active : inactive;
                const asDoc = (gap) => (gap === true ? " " : gap || "");
                return b.ifBreak(
                  asDoc(frame.first ? active : inactive),
                  asDoc(frame.first ? inactive : active),
                  { groupId: group },
                );
              };
              // Current-mode conditions participate in Prettier's fit probe;
              // named group modes are only available after a group is printed.
              append(
                choose(doc, "", frame.useCurrentMode ? undefined : frame.group),
              );
              // A deferred gap can leave this group before being consumed, so
              // keep its condition bound to the original logical group.
              pendingSpace = choose(
                pendingSpace,
                frame.pendingSpace,
                frame.group,
              );
              rawSpace ||= frame.rawSpace;
            }
            break;
          case 11:
            append(b.fill(frame.parts));
            break;
          case 12:
          case 13:
            if (stack.at(-1).op !== 11) throw invalid();
            append(doc);
            modeMatchesGroup = frame.modeMatchesGroup;
            break;
          case 14:
            append(b.lineSuffix(doc));
            pendingSpace = frame.pendingSpace;
            rawSpace = frame.rawSpace;
            break;
          default:
            throw invalid();
        }
        break;
      }
      case 18:
        if (first === 0) throw invalid();
        append(b.group("", { id: groupId(first) }));
        break;
      default:
        throw invalid();
    }
  }
  if (stack.length !== 1) throw invalid();
  return { doc: contents(stack[0].parts), pendingSpace, rawSpace };
}

// Canonical fitting ignores source-only expansion while retaining required lines.
function canonicalView(root, { sourceGroups, cache }) {
  if (typeof root === "string") return root;
  const work = [{ doc: root, finish: false }];
  const mapped = (doc) => (typeof doc === "string" ? doc : cache.get(doc));
  while (work.length) {
    const frame = work.pop();
    const doc = frame.doc;
    if (typeof doc === "string" || cache.has(doc)) continue;
    let children;
    if (Array.isArray(doc)) children = doc;
    else if (doc.type === "group" && doc.break && !sourceGroups.has(doc)) {
      cache.set(doc, doc);
      continue;
    } else if (doc.type === "line") {
      cache.set(doc, doc.hard ? doc : doc.soft ? "" : " ");
      continue;
    } else if (doc.type === "if-break") {
      children = doc.groupId
        ? [doc.breakContents, doc.flatContents]
        : [doc.flatContents];
    } else if (doc.type === "fill") children = doc.parts;
    else if (doc.contents !== undefined) children = [doc.contents];
    else {
      cache.set(doc, doc);
      continue;
    }
    if (!frame.finish) {
      work.push({ doc, finish: true });
      for (let index = children.length - 1; index >= 0; index--)
        work.push({ doc: children[index], finish: false });
      continue;
    }
    let result;
    if (Array.isArray(doc)) result = children.map(mapped);
    else if (doc.type === "group")
      result = b.group(mapped(doc.contents), { id: doc.id });
    else if (doc.type === "if-break")
      result = doc.groupId
        ? b.ifBreak(mapped(doc.breakContents), mapped(doc.flatContents), {
            groupId: doc.groupId,
          })
        : mapped(doc.flatContents);
    else if (doc.type === "fill") result = b.fill(children.map(mapped));
    else result = { ...doc, contents: mapped(doc.contents) };
    cache.set(doc, result);
  }
  return cache.get(root);
}
