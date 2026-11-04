function f() { a(); if (b) c(); }
function g() { a(); return b; }
function h() { a(); return; }
function i() { a(); throw b; }
function j() { a(); switch (b) {} }
function k() { a(); b(); return c, d; }
