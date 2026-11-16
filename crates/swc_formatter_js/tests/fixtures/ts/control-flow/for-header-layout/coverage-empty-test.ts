function f(): void {
 for (let i=0;;
 // @ts-expect-error
 missingUpdate
 ) {}
 const outside: number = "outside";
}
