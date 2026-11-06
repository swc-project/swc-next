function f() { a(); return; }
function g() { if (a) return; b(); c(); }
function h() { if (a) return; let b = c(); d(b); }
function i() { if (a) return; b(); if (c) return; d(); }
function j() { if (a) return; }
const k = () => { if (a) return; b(); };
