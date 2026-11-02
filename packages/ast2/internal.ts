// Shared AST metadata for package integrations. Keep the layout and projection
// tables generated from the same AST declarations as the runtime views.
export { NONE } from "./layout.ts";
export { NodeKind, layouts } from "./schema.ts";
export { fieldRule, hasEmptyDecorators, listDescriptor, fillList, isEmptyListDescriptor, readChildId, readChildIdWithRule, isProjectedChildRule } from "./projection.ts";
export {
  NODE_KIND_COUNT,
  FIELD_COUNT,
  CURSOR_FIELD_COUNT,
  ReadRule,
  cursorFieldIds,
  cursorRules,
  cursorRuleIds,
  fieldIds,
  fieldRules,
  fieldRuleIds,
  kindFlags,
  staticTypes,
  dynamicTypes,
} from "./protocol.ts";
