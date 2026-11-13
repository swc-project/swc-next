declare function f(): {};
// @ts-expect-error
const { alpha, beta: renamed,/* note */
 gamma } = f();
