function f() { use(a); var a = 1; return a; }
function g() { if (x) { var a = 1; } return a; }
function h() { { const a = 1; use(a); } }
