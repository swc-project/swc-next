function f(a = void b, b = 1) { void a; void b; }
function g(a, b = void a) { a; b; }
var h = (a = typeof b, b) => { a; b; };
try {} catch ({ a = void b, b = 1 }) { a; b; }
try {} catch (e) { e; }
