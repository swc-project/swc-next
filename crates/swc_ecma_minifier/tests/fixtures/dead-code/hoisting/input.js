if (0) { var a = f(), [b, { c, ...d }] = g(); for (var e in o); for (var p of o); }
use(a, b, c, d, e, p);
if (1) { x(); } else { var h; var h; }
use(h);
if (0) { var i = function () { var inner; }; }
if (0) var j = () => { var inner; };
if (0) { class C { static { var inner; } } }
use(i, j);
if (1) { let m; } else { var m; }
use(m);
if (0) { if (1) { var n; } else { var q; } }
use(n, q);
function scope() { if (0) { var unused = 1, used; } return used; }
