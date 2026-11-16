function f(value) {}
// @ts-expect-error
f(missing,
/* b */ // c
);
