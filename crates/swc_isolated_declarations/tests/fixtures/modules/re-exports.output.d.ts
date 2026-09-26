export type { Foo } from "./foo";
export { type Foo, Bar } from "./foo";
export { "default" as Foo } from "./foo";
export { Foo as "non-identifier" } from "./foo";
export { data } from "./data.json" with { type: "json" };
export * from "./data.json" with { type: "json" };
export * as data from "./data.json" with { type: "json" };
