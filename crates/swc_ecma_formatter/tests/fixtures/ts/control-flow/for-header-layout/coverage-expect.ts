function f(): void {
 for (
 // @ts-expect-error
 missingInit;
 missingTest;
 missingUpdate
 ) {}
 const outside: number = "outside";
}
