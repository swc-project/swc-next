async function f() { for (await /* head */ using resource = acquire(); /* middle */; /* tail */) release(); }
