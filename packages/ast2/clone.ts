import type { AstView } from "./index.ts";
import { layouts, NodeKind } from "./schema.ts";
import { NodeView } from "./node.ts";
import { fieldLocation, newNodeRaw } from "./storage.ts";
import { writeNativeFieldAt, writeNodeIdAt, writeReferencesAt } from "./writer.ts";
import { NONE } from "./layout.ts";

/**
 * Deep-clone a native node from `source` into `dest`.
 *
 * Fields are read through the native layout, so projected wrappers, enums and
 * source-backed strings are copied without materializing the source subtree.
 * Spans are copied only when both views share the same source text; otherwise
 * cloned nodes start at offset 0 so they never point at unrelated text. AST
 * metadata such as `Program.sourceType` belongs to the destination and is not
 * copied.
 */
export function cloneNode(dest: AstView, source: NodeView): NodeView {
  const sourceAst = source.ast;
  const preserveSpans = sourceAst.source === dest.source;
  // A clone is one logical edit; invalidate caches once instead of per field.
  dest._editVersion++;
  dest._listVersion++;
  return dest.nativeNode(
    cloneNodeAt(dest, sourceAst, source.nodeId, sourceAst.kind(source.nodeId), preserveSpans),
  );
}

function cloneNodeAt(dest: AstView, sourceAst: AstView, sourceId: number, kind: number,
  preserveSpans: boolean): number {
  const targetId = newNodeRaw(
    dest,
    kind,
    preserveSpans ? sourceAst.byteStart(sourceId) : 0,
    preserveSpans ? sourceAst.byteEnd(sourceId) : 0,
    false,
  );
  const fields = layouts[kind].fields;
  for (const key in fields) {
    if (!Object.hasOwn(fields, key)) { continue; }
    const descriptor = fields[key], type = descriptor[2];
    if (type === "node") {
      const { view, offset } = fieldLocation(sourceAst, sourceId, descriptor, kind);
      const childId = sourceAst.childAt(sourceId, offset, descriptor[3], view);
      const child = childId === NONE
        ? NONE : cloneNodeAt(dest, sourceAst, childId, sourceAst._kindAt(childId), preserveSpans);
      writeNodeIdAt(dest, targetId, kind, descriptor, child);
    } else if (type === "list") {
      const { view, offset } = fieldLocation(sourceAst, sourceId, descriptor, kind);
      const start = view.getUint32(offset, true), end = view.getUint32(offset + 4, true);
      if (start > end || end > sourceAst.extraDataCount) { throw new RangeError("AST list out of bounds"); }
      const ids = new Array<number>(end - start);
      for (let childIndex = 0; childIndex < ids.length; childIndex++) {
        const childId = sourceAst.extraChild(sourceId, start + childIndex, true);
        ids[childIndex] = childId === NONE
          ? NONE : cloneNodeAt(dest, sourceAst, childId, sourceAst._kindAt(childId), preserveSpans);
      }
      writeReferencesAt(dest, targetId, kind, descriptor, ids);
    } else if (kind === NodeKind.TemplateElement && key === "raw") {
      // Parser-backed TypeScript template raw is projected from the source
      // rather than stored, so copy the projected value into the new node.
      const value = (sourceAst.getField(sourceId, "value", kind) as unknown as { raw: string }).raw;
      writeNativeFieldAt(dest, targetId, kind, key, descriptor, value, false);
    } else {
      const value = sourceAst._valueAt(sourceId, key, descriptor, kind);
      writeNativeFieldAt(dest, targetId, kind, key, descriptor, value, false);
    }
  }
  return targetId;
}
