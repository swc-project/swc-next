const key = "p";
interface I {
  method({ [key]: v }: { p: number }): typeof v;
}
