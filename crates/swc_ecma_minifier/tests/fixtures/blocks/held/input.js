a(); { b(); { c(); var d; { var e; } } }
try { a(); { b(); } } catch (x) { c(); { d(); } } finally { e(); { f(); } }
while (x) { a(); { b(); } }
if (x) { a(); { b(); } } else { c(); { d(); } }
while (x) if (1) { a(); { b(); } }
{ let l = 1; a(); { b(); } }
