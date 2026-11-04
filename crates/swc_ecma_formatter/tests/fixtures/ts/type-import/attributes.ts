type A=import("./m",{with:{"resolution-mode":"import"}}).Value;

type MultilineOptions=import("./m", {
 with:{"resolution-mode":"import"}
}).Value;
type NestedOptions=import("./m", {with:{
 "resolution-mode":"import"
}}).Value;
type QualifiedOptions=import("./m",{with:{"resolution-mode":"import"}}).Generic<string,number>;
type CommentedOptions=import("./m", /* options */ {with:{"resolution-mode":"import"}}).Value;
type IgnoredOptions=import("./m",
 // prettier-ignore
 {with:{"resolution-mode": "import"}}
).Value;
type EmptyOptions=import("./m",{});
