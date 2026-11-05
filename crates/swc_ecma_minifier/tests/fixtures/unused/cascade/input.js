function f() { var a = {}; var b = function () { return a; }; function g() { return b; } }
function h() { var c = {}; function i() { return c; } return i; }
