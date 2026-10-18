import { builders as b } from "prettier/doc";

// Undefined is statically Flat, true is Break, and Symbols bind a host group.
// Null queries the current print mode rather than a named logical group.
// Persistent, interned 32-bit tries keep context projections and effect overlays
// bounded by wire IDs rather than copying maps at every shared reference.
export class GroupBindings {
  constructor(fragments, order) {
    this.tracked = new Set();
    const defined = new Set();
    for (const ops of fragments)
      for (const [op, first, second] of ops) {
        if (op === 3 && second) defined.add(second);
        if (op === 7 || op === 18) this.tracked.add(first);
        if (op === 8 && second > 0 && second < 0xffffffff)
          this.tracked.add(second);
      }
    this.tracked = new Set([...this.tracked].filter((id) => defined.has(id)));
    this.nodes = new Map();
    this.projections = new Map();
    this.unions = new Map();
    this.conditions = new Map();
    this.rendered = new Map();
    this.footprints = [];
    for (const id of order) {
      let mask;
      for (const [op, first, second] of fragments[id]) {
        if (op === 9) mask = this.union(mask, this.footprints[first]);
        else if (op === 10 || op === 20) {
          for (const child of first)
            mask = this.union(mask, this.footprints[child]);
        } else {
          const group =
            op === 3 || op === 8 ? second : op === 7 || op === 18 ? first : 0;
          if (this.tracked.has(group)) mask = this.set(mask, group, true);
        }
      }
      this.footprints[id] = mask;
    }
  }

  node(left, right) {
    if (left === undefined && right === undefined) return undefined;
    if (!this.nodes.has(left)) this.nodes.set(left, new Map());
    const children = this.nodes.get(left);
    if (!children.has(right)) children.set(right, { left, right });
    return children.get(right);
  }

  set(root, id, value, bit = 31) {
    if (bit < 0) return value;
    return (id >>> bit) & 1
      ? this.node(root?.left, this.set(root?.right, id, value, bit - 1))
      : this.node(this.set(root?.left, id, value, bit - 1), root?.right);
  }

  get(root, id) {
    for (let bit = 31; bit >= 0 && root !== undefined; bit--)
      root = (id >>> bit) & 1 ? root.right : root.left;
    return root;
  }

  define(root, id, value) {
    return this.tracked.has(id) ? this.set(root, id, value) : root;
  }

  writes(mask, id) {
    return this.tracked.has(id) ? this.set(mask, id, true) : mask;
  }

  union(left, right, bit = 31) {
    if (left === right || right === undefined) return left;
    if (left === undefined) return right;
    if (bit < 0) return true;
    if (!this.unions.has(left)) this.unions.set(left, new Map());
    const cache = this.unions.get(left);
    if (!cache.has(right))
      cache.set(
        right,
        this.node(
          this.union(left.left, right.left, bit - 1),
          this.union(left.right, right.right, bit - 1),
        ),
      );
    return cache.get(right);
  }

  project(root, mask, bit = 31) {
    if (root === undefined || mask === undefined) return undefined;
    if (bit < 0) return root;
    if (!this.projections.has(root)) this.projections.set(root, new Map());
    const cache = this.projections.get(root);
    if (!cache.has(mask))
      cache.set(
        mask,
        this.node(
          this.project(root.left, mask.left, bit - 1),
          this.project(root.right, mask.right, bit - 1),
        ),
      );
    return cache.get(mask);
  }

  apply(root, changes, mask, bit = 31) {
    if (mask === undefined || root === changes) return root;
    if (bit < 0) return changes;
    return this.node(
      this.apply(root?.left, changes?.left, mask.left, bit - 1),
      this.apply(root?.right, changes?.right, mask.right, bit - 1),
    );
  }

  condition(predicate, active, inactive) {
    if (active === inactive) return active;
    if (predicate.binding === true) return predicate.broken ? active : inactive;
    if (predicate.binding === undefined)
      return predicate.broken ? inactive : active;
    if (active === predicate.binding)
      active = predicate.broken ? true : undefined;
    if (inactive === predicate.binding)
      inactive = predicate.broken ? undefined : true;
    if (active === inactive) return active;
    if (!this.conditions.has(predicate.binding))
      this.conditions.set(predicate.binding, new Map());
    let cache = this.conditions.get(predicate.binding);
    for (const key of [predicate.broken, active]) {
      if (!cache.has(key)) cache.set(key, new Map());
      cache = cache.get(key);
    }
    if (!cache.has(inactive))
      cache.set(inactive, { predicate, active, inactive });
    return cache.get(inactive);
  }

  merge(predicate, active, inactive, bit = 31) {
    if (active === inactive) return active;
    if (predicate.binding === true) return predicate.broken ? active : inactive;
    if (predicate.binding === undefined)
      return predicate.broken ? inactive : active;
    if (bit < 0) return this.condition(predicate, active, inactive);
    return this.node(
      this.merge(predicate, active?.left, inactive?.left, bit - 1),
      this.merge(predicate, active?.right, inactive?.right, bit - 1),
    );
  }

  // Render predicates with public Docs. Jobs carry their branch contents so
  // nested binding predicates never recurse through the JavaScript call stack.
  select(binding, flat, broken) {
    if (flat === broken || binding === undefined) return flat;
    if (binding === true) return broken;
    const cacheFor = (flat, broken) => {
      if (!this.rendered.has(flat)) this.rendered.set(flat, new Map());
      const cache = this.rendered.get(flat);
      if (!cache.has(broken)) cache.set(broken, new Map());
      return cache.get(broken);
    };
    const primitive = (binding) =>
      binding === undefined ||
      binding === true ||
      binding === null ||
      typeof binding === "symbol";
    const value = (binding, flat, broken) => {
      if (flat === broken || binding === undefined) return flat;
      if (binding === true) return broken;
      if (binding === null || typeof binding === "symbol") {
        const cache = cacheFor(flat, broken);
        if (!cache.has(binding))
          cache.set(
            binding,
            b.ifBreak(broken, flat, { groupId: binding ?? undefined }),
          );
        return cache.get(binding);
      }
      return cacheFor(flat, broken).get(binding);
    };
    if (primitive(binding)) return value(binding, flat, broken);
    const work = [{ binding, flat, broken, phase: 0 }];
    while (work.length) {
      const frame = work.at(-1);
      const { binding: node, flat, broken } = frame;
      if (flat === broken || primitive(node)) {
        work.pop();
        continue;
      }
      const cache = cacheFor(flat, broken);
      if (cache.has(node)) {
        work.pop();
        continue;
      }
      if (frame.phase === 0) {
        frame.phase = 1;
        const children = [node.active, node.inactive];
        for (const binding of children)
          work.push({ binding, flat, broken, phase: 0 });
      } else if (frame.phase === 1) {
        const { predicate } = node;
        const active = value(node.active, flat, broken);
        const inactive = value(node.inactive, flat, broken);
        frame.flatBranch = predicate.broken ? inactive : active;
        frame.brokenBranch = predicate.broken ? active : inactive;
        frame.phase = 2;
        work.push({
          binding: predicate.binding,
          flat: frame.flatBranch,
          broken: frame.brokenBranch,
          phase: 0,
        });
      } else {
        cache.set(
          node,
          value(node.predicate.binding, frame.flatBranch, frame.brokenBranch),
        );
        work.pop();
      }
    }
    return value(binding, flat, broken);
  }
}
