switch (value) {
  case 0: /* first */ { break; }
  default: /* last */
    { break; }
}

switch (x) { case 1: if (a) b(); // c
case 2: break; }

switch (x) { default: if (a) b();; /* c */
}

switch (x) { case 1:
// @ts-ignore
f(); /* prettier-ignore */; default: g(); }
