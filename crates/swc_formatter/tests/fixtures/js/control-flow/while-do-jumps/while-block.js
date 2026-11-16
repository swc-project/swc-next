while (test()) {if (ready) break; continue;}

while (false) {}

for (; false;) {}

do {} while (false);

while (false) { globalThis.loopTrace.push("never"); }

for (; false;) { globalThis.loopTrace.push("never"); }

do { globalThis.loopTrace.push("once"); } while (false);

while (false) { "use strict"; }

while (false) { /* dangling */ }

for (; false;) { // dangling line
}

do { /* dangling */ } while (false);

while (false)
// prettier-ignore
{   }

// prettier-ignore
while( false ){   }

while (false) { /* prettier-ignore */ }

if (false) {} else {}

for (const value of []) {}

for (const key in {}) {}

with ({}) {}

label: {}

while (false) nested: {}

{}
