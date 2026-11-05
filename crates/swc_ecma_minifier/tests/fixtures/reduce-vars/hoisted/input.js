function f() { var a = 1; function g() { return a; } return g; }
function h() { var a = 1; return function () { return a; }; }
