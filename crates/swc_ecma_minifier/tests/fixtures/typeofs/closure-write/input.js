function f() { var x = 1; function g() { x = source; } return [g, typeof x === "undefined"]; }
