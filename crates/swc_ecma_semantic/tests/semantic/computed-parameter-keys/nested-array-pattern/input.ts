const key = "p";
type F = ([
  {
    nested: { [key]: v },
  },
]: [{ nested: { p: number } }]) => typeof v;
