function f() {
// @ts-expect-error
return missingLongName +
otherLongName;
}
