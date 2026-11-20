const key = "p";
function f({ [key]: v }: { p: number }) {
  return v;
}
