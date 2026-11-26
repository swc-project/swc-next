import type { AstView } from "#protocol/index";
import { fieldDescriptors, layouts } from "#protocol/schema";
import { NodeView } from "#protocol/node";
import { fieldLocation, newNodeRaw } from "#protocol/storage";
import { writeNativeFieldAt, writeNodeIdAt, writeReferencesAt } from "#protocol/writer";
import { NONE } from "#protocol/layout";

/**
 * Deep-clone a native node from `source` into `dest`.
 *
 * Fields are read through the native layout, so nodes, enums and
 * source-backed strings are copied without materializing the source subtree.
 * Spans are copied only when both views share the same source text; otherwise
 * cloned nodes start at offset 0 so they never point at unrelated text. AST
 * metadata such as the source type belongs to the destination and is not
 * copied.
 */
export function cloneNode(dest: AstView, source: NodeView): NodeView {
  const sourceAst = source.ast;
  const preserveSpans = sourceAst.source === dest.source;
  // A clone is one logical edit; invalidate caches once instead of per field.
  dest._editVersion++;
  dest._listVersion++;
  return dest.node(
    cloneNodeAt(dest, sourceAst, source.nodeId, sourceAst.kind(source.nodeId), preserveSpans),
  )!;
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
  for (const [field, descriptorId] of layouts[kind].fields) {
    const descriptor = fieldDescriptors[descriptorId]!, type = descriptor[2];
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
    } else {
      const value = sourceAst._valueAt(sourceId, field, descriptor, kind);
      writeNativeFieldAt(dest, targetId, kind, field, descriptor, value, false);
    }
  }
  return targetId;
}
