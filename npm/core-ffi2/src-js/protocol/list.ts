import type { AstView } from "#protocol/index";
import type { AstNode, NodeDescriptor } from "#protocol/types";
import type { NodeView } from "#protocol/node";
import { fieldNames } from "#protocol/protocol";

export type ListEntry = NodeView | null | undefined;


/** A live AST field. The view retains positions, never copies of its elements. */
export class AstList<T = AstNode> implements Iterable<T> {
  // Keep the non-extensible receiver shape alive without retaining AST storage.
  static #shape = new this(null!, 0, 0);
  readonly ast: AstView;
  readonly owner: number;
  readonly field: number;
  #start = 0;
  #end = 0;
  #version = -1;

  constructor(ast: AstView, owner: number, field: number) {
    this.ast = ast; this.owner = owner; this.field = field;
    Object.preventExtensions(this);
  }
  get key(): string { return fieldNames[this.field]; }
  refresh() {
    if (this.#version !== this.ast._listVersion) {
      [this.#start, this.#end] = this.ast._range(this.owner, this.field);
      this.#version = this.ast._listVersion;
    }
  }
  get length(): number {
    this.refresh();
    return this.#end - this.#start;
  }
  nodeId(index: number): number {
    if (!Number.isInteger(index) || index < 0 || index >= this.length) { throw new RangeError("AST list index out of bounds"); }
    return this.#nodeId(index);
  }
  #nodeId(index: number): number {
    return this.ast.extraChild(this.owner, this.#start + index, true);
  }
  get(index: number): T | undefined {
    if (!Number.isInteger(index) || index < 0 || index >= this.length) { return; }
    return this.#getAt(index);
  }
  #getAt(index: number): T {
    return this.ast._nodeAt(this.#nodeId(index)) as T;
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
    this.ast.writeNativeField(this.owner, this.field, input);
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
    && left.ast === right.ast && left.owner === right.owner && left.field === right.field;
}
function integer(value: number): number {
  const number = Number(value);
  return Number.isNaN(number) ? 0 : Math.trunc(number);
}

function position(value: number, length: number): number {
  const index = integer(value);
  return index < 0 ? Math.max(length + index, 0) : Math.min(index, length);
}
