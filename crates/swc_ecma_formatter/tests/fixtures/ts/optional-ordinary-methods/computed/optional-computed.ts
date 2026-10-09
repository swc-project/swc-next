const key = "read"; class OptionalComputed { value = 1; [key]?(): number { return this.value + 6; } }
const result = new OptionalComputed()[key]?.();
