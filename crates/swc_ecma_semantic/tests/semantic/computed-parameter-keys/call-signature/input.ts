const key = "p";
interface I {
  ({ [key]: v }: { p: number }): typeof v;
}
