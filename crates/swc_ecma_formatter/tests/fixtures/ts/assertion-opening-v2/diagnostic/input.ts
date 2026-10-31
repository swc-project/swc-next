type A = string;
type B = boolean;
const x =
// @ts-expect-error
</* a */ /* b */A | B>123;
