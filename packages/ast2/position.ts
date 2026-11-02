/**
 * Sparse UTF-8 to UTF-16 position index with a cached run.
 *
 * The cache lives for the whole AST view. Every field is initialized in the
 * constructor so the class keeps one hidden class; the hot methods are
 * monomorphic and do not allocate.
 */
const EMPTY_VIEW = new DataView(new ArrayBuffer(8));

export class PositionCache {
  view: DataView = EMPTY_VIEW;
  runs = 0;
  count = 0;
  excess = 0;
  sourceLength = 0;
  sourceUnits = 0;
  first = 0;
  firstUnit = 0;
  index = -1;
  runStart = 0;
  runEnd = 0;
  runExcess = 0;
  width = 0;

  constructor(view: DataView, positionsInfo: number, sourceLength: number) {
    const count = view.getUint32(positionsInfo, true);
    const excess = view.getUint32(positionsInfo + 4, true);
    const runs = positionsInfo + 8;
    this.view = view;
    this.runs = runs;
    this.count = count;
    this.excess = excess;
    this.sourceLength = sourceLength;
    this.sourceUnits = sourceLength - excess;
    this.first = count === 0 ? 0 : view.getUint32(runs, true);
    this.firstUnit = count === 0 ? 0
      : view.getUint32(runs, true) - view.getUint32(runs + 4, true);
  }

  /** Query the Rust-built source index without scanning or decoding source bytes. */
  utf16Offset(byte: number): number {
    if (!Number.isInteger(byte) || byte < 0)
      { throw new RangeError("Invalid source byte offset"); }
    const sourceLength = this.sourceLength;
    if (byte > sourceLength || this.count === 0) { return byte; }
    if (byte === sourceLength) { return this.sourceUnits; }
    if (byte <= this.first) { return byte; }
    const index = this.index;
    if (index >= 0) {
      const start = this.runStart, end = this.runEnd, excess = this.runExcess, width = this.width;
      if (byte >= start) {
        if (byte < end) { return this._utf16InRun(byte, start, excess, width); }
        const excessAfter = excess + (end - start) / width * (width === 4 ? 2 : width - 1);
        const next = index + 1;
        if (next >= this.count) { return byte - excessAfter; }
        if (byte < this.view.getUint32(this.runs + next * 16, true)) {
          return byte - excessAfter;
        }
      }
    }
    return this._utf16Slow(byte);
  }

  /** Convert modified ESTree positions back to the original source's UTF-8 offsets. */
  utf8Offset(position: number): number {
    if (!Number.isInteger(position) || position < 0)
      { throw new RangeError("Invalid UTF-16 source offset"); }
    const sourceUnits = this.sourceUnits;
    if (position > sourceUnits)
      { throw new RangeError("AST span is outside the original source"); }
    if (this.count === 0 || position === 0) { return position; }
    if (position === sourceUnits) { return this.sourceLength; }
    if (position <= this.firstUnit) { return position; }
    const index = this.index;
    if (index >= 0) {
      const start = this.runStart, end = this.runEnd, excess = this.runExcess, width = this.width;
      const units = start - excess, unitWidth = width === 4 ? 2 : 1;
      if (position >= units) {
        const unitEnd = units + (end - start) / width * unitWidth;
        if (position < unitEnd) { return this._utf8InRun(position, start, excess, width); }
        const excessAfter = excess + (end - start) / width * (width - unitWidth);
        const next = index + 1;
        if (next >= this.count) { return position + excessAfter; }
        const nextAt = this.runs + next * 16;
        const nextUnits = this.view.getUint32(nextAt, true) - this.view.getUint32(nextAt + 4, true);
        if (position < nextUnits) { return position + excessAfter; }
      }
    }
    return this._utf8Slow(position);
  }

  _utf16Slow(byte: number): number {
    const index = this.index;
    if (index >= 0) {
      let start = this.runStart, end = this.runEnd, excess = this.runExcess, width = this.width;
      if (byte >= start) {
        let excessAfter = excess + (end - start) / width * (width === 4 ? 2 : width - 1);
        const next = index + 1;
        if (next >= this.count) { return byte - excessAfter; }
        const limit = Math.min(next + 8, this.count);
        let cursor = next;
        while (cursor < limit) {
          this._load(cursor);
          start = this.runStart; end = this.runEnd;
          excess = this.runExcess; width = this.width;
          if (byte < start) { return byte - excessAfter; }
          if (byte < end) { return this._utf16InRun(byte, start, excess, width); }
          excessAfter = excess + (end - start) / width * (width === 4 ? 2 : width - 1);
          cursor++;
        }
      }
    }
    if (this._findRun(byte) < 0) { return byte; }
    const start = this.runStart, end = this.runEnd, excess = this.runExcess, width = this.width;
    if (byte < end) { return this._utf16InRun(byte, start, excess, width); }
    return byte - excess - (end - start) / width * (width === 4 ? 2 : width - 1);
  }

  _utf8Slow(position: number): number {
    const index = this.index;
    if (index >= 0) {
      let start = this.runStart, end = this.runEnd, excess = this.runExcess, width = this.width;
      let units = start - excess, unitWidth = width === 4 ? 2 : 1;
      if (position >= units) {
        let excessAfter = excess + (end - start) / width * (width - unitWidth);
        const next = index + 1;
        if (next >= this.count) { return position + excessAfter; }
        const limit = Math.min(next + 8, this.count);
        let cursor = next;
        while (cursor < limit) {
          this._load(cursor);
          start = this.runStart; end = this.runEnd;
          excess = this.runExcess; width = this.width;
          units = start - excess; unitWidth = width === 4 ? 2 : 1;
          if (position < units) { return position + excessAfter; }
          const unitEnd = units + (end - start) / width * unitWidth;
          if (position < unitEnd) { return this._utf8InRun(position, start, excess, width); }
          excessAfter = excess + (end - start) / width * (width - unitWidth);
          cursor++;
        }
      }
    }
    if (this._findUnit(position) < 0) { return position; }
    const start = this.runStart, end = this.runEnd, excess = this.runExcess, width = this.width;
    const units = start - excess, unitWidth = width === 4 ? 2 : 1;
    const unitEnd = units + (end - start) / width * unitWidth;
    if (position < unitEnd) { return this._utf8InRun(position, start, excess, width); }
    return position + excess + (end - start) / width * (width - unitWidth);
  }

  _load(index: number): void {
    const view = this.view, at = this.runs + index * 16;
    const start = view.getUint32(at, true), excess = view.getUint32(at + 4, true),
      width = view.getUint32(at + 8, true), end = view.getUint32(at + 12, true);
    if (width < 2 || width > 4 || start >= end || end > this.sourceLength ||
        excess > start || (end - start) % width)
      { throw new RangeError("Invalid source position run"); }
    this.index = index;
    this.runStart = start;
    this.runEnd = end;
    this.runExcess = excess;
    this.width = width;
  }

  _findRun(byte: number): number {
    const view = this.view, table = this.runs;
    let low = 0, high = this.count;
    while (low < high) {
      const mid = (low + high) >>> 1;
      if (view.getUint32(table + mid * 16, true) <= byte) { low = mid + 1; }
      else { high = mid; }
    }
    if (low === 0) { this.index = -1; return -1; }
    this._load(low - 1);
    return low - 1;
  }

  _findUnit(position: number): number {
    const view = this.view, table = this.runs;
    let low = 0, high = this.count;
    while (low < high) {
      const mid = (low + high) >>> 1, at = table + mid * 16;
      const units = view.getUint32(at, true) - view.getUint32(at + 4, true);
      if (units <= position) { low = mid + 1; }
      else { high = mid; }
    }
    if (low === 0) { this.index = -1; return -1; }
    this._load(low - 1);
    return low - 1;
  }

  _utf16InRun(byte: number, start: number, excess: number, width: number): number {
    const distance = byte - start;
    return start - excess + Math.floor(distance / width) * (width === 4 ? 2 : 1) +
      (width === 4 && distance % width === 3 ? 1 : 0);
  }

  _utf8InRun(position: number, start: number, excess: number, width: number): number {
    const distance = position - (start - excess);
    return start + (width === 4
      ? Math.floor(distance / 2) * 4 + distance % 2 * 3
      : distance * width);
  }
}
