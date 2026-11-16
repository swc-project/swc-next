function f() {
// @ts-expect-error
throw missingLongName + otherLongName;
}
