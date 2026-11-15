function f() { return true && (1 & 1 // @ts-expect-error
); missingCovered(); }
missingUncovered();
