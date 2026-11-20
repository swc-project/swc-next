if (true) String(0); /* @ts-ignore */ else String(1);
const count: number = "zero";

do String(0); /* @ts-expect-error */ while (false);
const flag: boolean = "true";

if (true) String(0) /* @ts-ignore */
; else missing();
