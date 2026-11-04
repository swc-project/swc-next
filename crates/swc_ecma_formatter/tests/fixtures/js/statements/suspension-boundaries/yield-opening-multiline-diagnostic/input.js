function* f() {
// @ts-expect-error
yield missingLongName +
otherLongName;
}
