class OptionalKeyIgnore { public /* prettier-ignore */ method?(): number { return 7; } }
const result = new OptionalKeyIgnore().method?.();
