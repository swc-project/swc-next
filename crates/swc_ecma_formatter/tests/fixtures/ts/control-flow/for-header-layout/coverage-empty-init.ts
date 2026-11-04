function f(): void {
 for (;
 // @ts-expect-error
 missingTest;
 missingUpdate
 ) {}
 const outside: number = "outside";
}
