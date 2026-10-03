async function f() { for await (await /* first */ using /* second */ resource of acquire()) use(resource); }
