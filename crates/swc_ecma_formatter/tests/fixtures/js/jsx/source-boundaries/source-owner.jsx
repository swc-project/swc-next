// @ts-ignore
const value0=<MissingPanel>{/* prettier-ignore */}<IgnoredChild odd = {missingAttribute}  ></IgnoredChild></MissingPanel>;

// @ts-ignore
const value1=<MissingPanel>{/* prettier-ignore */}<><IgnoredChild odd = {missingAttribute}  /></></MissingPanel>;

const value2=<MissingPanel>{/* prettier-ignore */ receiver.a(// @ts-ignore
missingIgnored(1)).b(2).c(3)}</MissingPanel>;

const value3=<MissingPanel>{(receiver.a(// @ts-ignore
missingPaired(1)).b(2).c(3))}</MissingPanel>;
