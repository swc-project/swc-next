const key = "p";
interface I {
  new ({ [key]: v }: { p: number }): object;
}
