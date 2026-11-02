import type { AstView } from "./index.ts";
import type { ListCursor, NodeDescriptor } from "./types.ts";
import type { NodeView } from "./node.ts";
import { NONE } from "./layout.ts";

export type ListEntry = NodeView | null | undefined;


/** A live AST field. The view retains positions, never copies of its elements. */
export class AstList<T = ListEntry> implements ListCursor, Iterable<T> {
  // Keep the non-extensible receiver shape alive without retaining AST storage.
  static #shape = new this(null!, 0, "");
  readonly ast: AstView;
  readonly owner: number;
  readonly key: string;
  readonly native: boolean;
  prefix = -1;
  suffix = -1;
  owner0 = 0;
  owner1 = 0;
  start0 = 0;
  end0 = 0;
  start1 = 0;
  end1 = 0;
  version = -1;
  private total = 0;

  constructor(ast: AstView, owner: number, key: string, native = false) {
    this.ast = ast; this.owner = owner; this.key = key; this.native = native;
    Object.preventExtensions(this);
  }
  setRange(part: number, owner: number, start: number, end: number) {
    if (part === 0) { this.owner0 = owner; this.start0 = start; this.end0 = end; }
    else { this.owner1 = owner; this.start1 = start; this.end1 = end; }
  }
  refresh() {
    if (this.version !== this.ast._listVersion) { this.#refreshRange(); }
  }
  #refreshRange() {
    this.prefix = this.suffix = -1;
    this.start0 = this.end0 = this.start1 = this.end1 = 0;
    if (this.native) {
      const [start, end] = this.ast.range(this.owner, this.key);
      this.setRange(0, this.owner, start, end);
    } else if (!this.ast.readListCursor(this.owner, this.key, this)) { throw new TypeError("Field is not an AST list"); }
    this.version = this.ast._listVersion;
    this.total = this.end0 - this.start0 + this.end1 - this.start1 + Number(this.prefix !== -1) + Number(this.suffix !== -1);
  }
  get length(): number {
    this.refresh();
    return this.total;
  }
  nodeId(index: number): number {
    if (!Number.isInteger(index) || index < 0 || index >= this.length) { throw new RangeError("AST list index out of bounds"); }
    return this.#nodeId(index);
  }
  #nodeId(index: number): number {
    if (this.prefix !== -1 || index >= this.end0 - this.start0) { return this.#projectedId(index); }
    return this.ast.extraChild(this.owner0, this.start0 + index, true);
  }
  #projectedId(index: number): number {
    if (this.prefix !== -1) { if (index === 0) { return this.prefix >>> 0; } index--; }
    const first = this.end0 - this.start0;
    if (index < first) { return this.ast.extraChild(this.owner0, this.start0 + index, true); }
    index -= first;
    if (index < this.end1 - this.start1) { return this.ast.extraChild(this.owner1, this.start1 + index, true); }
    return this.suffix >>> 0;
  }
  get(index: number): T | undefined {
    if (!Number.isInteger(index) || index < 0 || index >= this.length) { return; }
    return this.#getAt(index);
  }
  #getAt(index: number): T {
    const id = this.#nodeId(index);
    return (id === NONE ? null : this.native ? this.ast.nativeNode(id) : this.ast._nodeAt(id)) as T;
  }
  set(index: number, value: T | NodeDescriptor): void {
    if (!Number.isInteger(index) || index < 0 || index >= this.length) { throw new RangeError("AST list index out of bounds"); }
    this.#write(new ListEdit(this, index, 1, [value]));
  }
  splice(start: number, deleteCount?: number, ...values: (T | NodeDescriptor)[]): T[] {
    const length = this.length, begin = position(start, length);
    const removed = arguments.length === 1 ? length - begin : Math.min(Math.max(integer(deleteCount ?? 0), 0), length - begin);
    const result = Array.from({ length: removed }, (_, index) => this.#getAt(begin + index));
    if (removed !== 0 || values.length !== 0) {
      const input = new ListEdit(this, begin, removed, values);
      this.#write(input);
    }
    return result;
  }
  #write(input: ListInput<T>): void {
    if (this.native) { this.ast.writeNativeField(this.owner, this.key, input); }
    else { this.ast.setField(this.owner, this.key, input); }
  }
  push(...values: (T | NodeDescriptor)[]): number {
    if (values.length) { this.#write(new ListEdit(this, this.length, 0, values)); }
    return this.length;
  }
  pop(): T | undefined { return this.length ? this.splice(this.length - 1, 1)[0] : undefined; }
  unshift(...values: (T | NodeDescriptor)[]): number {
    if (values.length) { this.#write(new ListEdit(this, 0, 0, values)); }
    return this.length;
  }
  shift(): T | undefined { return this.length ? this.splice(0, 1)[0] : undefined; }
  clear(): void { if (this.length) { this.#write([]); } }
  replace(values: Iterable<T | NodeDescriptor>): void {
    this.#write(values instanceof AstList || Array.isArray(values) ? values : Array.from(values));
  }
  reverse(): this { this.replace(this.toArray().reverse()); return this; }
  sort(compare?: (a: T, b: T) => number): this { this.replace(this.toArray().sort(compare)); return this; }
  slice(start = 0, end = this.length): T[] {
    const length = this.length, begin = position(start, length), finish = position(end, length);
    return Array.from({ length: Math.max(0, finish - begin) }, (_, index) => this.#getAt(begin + index));
  }
  toArray(): T[] { return this.slice(); }
  map<R>(callback: (value: T, index: number, list: this) => R): R[] {
    const length = this.length, result: R[] = new Array(length);
    for (let index = 0; index < length; index++) { result[index] = callback(this.#getAt(index), index, this); }
    return result;
  }
  filter(callback: (value: T, index: number, list: this) => unknown): T[] {
    const result: T[] = [];
    for (let index = 0, length = this.length; index < length; index++) {
      const value = this.#getAt(index);
      if (callback(value, index, this)) { result.push(value); }
    }
    return result;
  }
  forEach(callback: (value: T, index: number, list: this) => void): void {
    for (let index = 0, length = this.length; index < length; index++) { callback(this.#getAt(index), index, this); }
  }
  some(callback: (value: T, index: number, list: this) => unknown): boolean {
    for (let index = 0, length = this.length; index < length; index++) { if (callback(this.#getAt(index), index, this)) { return true; } }
    return false;
  }
  every(callback: (value: T, index: number, list: this) => unknown): boolean {
    for (let index = 0, length = this.length; index < length; index++) { if (!callback(this.#getAt(index), index, this)) { return false; } }
    return true;
  }
  find(callback: (value: T, index: number, list: this) => unknown): T | undefined {
    for (let index = 0, length = this.length; index < length; index++) { const value = this.#getAt(index); if (callback(value, index, this)) { return value; } }
  }
  findIndex(callback: (value: T, index: number, list: this) => unknown): number {
    for (let index = 0, length = this.length; index < length; index++) { if (callback(this.#getAt(index), index, this)) { return index; } }
    return -1;
  }
  toJSON(): T[] { return this.toArray(); }
  /** Internal iterator hook; the iterator class lives outside the private scope. */
  _getAt(index: number): T { return this.#getAt(index); }
  [Symbol.iterator](): IterableIterator<T> { return new AstListIterator(this); }
}

/** Plain iterator object; generators deopt and allocate more in hot `for...of` loops. */
class AstListIterator<T = ListEntry> implements IterableIterator<T> {
  #list: AstList<T>;
  #length: number;
  #index = 0;
  constructor(list: AstList<T>) { this.#list = list; this.#length = list.length; }
  next(): IteratorResult<T> {
    if (this.#index >= this.#length) { return { value: undefined, done: true }; }
    return { value: this.#list._getAt(this.#index++), done: false };
  }
  [Symbol.iterator](): IterableIterator<T> { return this; }
}

/** One synchronous splice input; existing elements remain numeric references. */
export class ListEdit<T = ListEntry> implements Iterable<T | NodeDescriptor> {
  readonly source: AstList<T>;
  readonly start: number;
  readonly removed: number;
  readonly inserted: readonly (T | NodeDescriptor)[];
  constructor(source: AstList<T>, start: number, removed: number, inserted: readonly (T | NodeDescriptor)[]) {
    this.source = source; this.start = start; this.removed = removed; this.inserted = inserted;
  }
  get length(): number { return this.source.length - this.removed + this.inserted.length; }
  nodeId(index: number): number | undefined {
    if (index < this.start) { return this.source.nodeId(index); }
    if (index < this.start + this.inserted.length) { return; }
    return this.source.nodeId(index - this.inserted.length + this.removed);
  }
  get(index: number): T | NodeDescriptor | undefined {
    if (index < this.start) { return this.source.get(index); }
    if (index < this.start + this.inserted.length) { return this.inserted[index - this.start]; }
    return this.source.get(index - this.inserted.length + this.removed);
  }
  *[Symbol.iterator](): IterableIterator<T | NodeDescriptor> { for (let index = 0; index < this.length; index++) { yield this.get(index)!; } }
}

export type ListInput<T = ListEntry> = readonly (T | NodeDescriptor)[] | AstList<T> | ListEdit<T>;

export function isList(value: unknown): value is ListInput {
  return Array.isArray(value) || value instanceof AstList || value instanceof ListEdit;
}

export function listAt<T>(values: ListInput<T> | null | undefined, index: number): T | NodeDescriptor | undefined {
  return values instanceof AstList || values instanceof ListEdit ? values.get(index) : values?.[index];
}

export function sameList(left: unknown, right: unknown): boolean {
  return left === right || left instanceof AstList && right instanceof AstList
    && left.ast === right.ast && left.owner === right.owner && left.key === right.key && left.native === right.native;
}
function integer(value: number): number {
  const number = Number(value);
  return Number.isNaN(number) ? 0 : Math.trunc(number);
}

function position(value: number, length: number): number {
  const index = integer(value);
  return index < 0 ? Math.max(length + index, 0) : Math.min(index, length);
}
