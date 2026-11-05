function f() { var x = 42; x = 43; { let x = 1; { function x() {} } use(x); } return x; }
function g() { var y = 1; { const y = 2; { function y() {} } use(y, y); } return y; }
function h() { { let z = 1; { if (a) function z() {} } use(z); } return z; }
function i() { { let w = 1; use(w); } { function w() {} } }
function j() { var v = 1; { function v() {} } return v; }
