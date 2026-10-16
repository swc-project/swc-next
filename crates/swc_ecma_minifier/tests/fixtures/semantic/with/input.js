function f() { var unused = 1; var x; with (source) { use(undefined, typeof x === "undefined", true); } }
