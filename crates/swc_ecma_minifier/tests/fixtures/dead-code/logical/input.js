use(1 && a, 0 && a, "" || a, "x" || a);
use(null ?? a, void 0 ?? a, 0 ?? a, "" ?? a);
use(1n && a, x && a, a && 0);
use((null ?? a) || b, (0 || a) ?? b);
!0 && run();
