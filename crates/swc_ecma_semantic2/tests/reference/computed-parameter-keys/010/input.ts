const key = "p";
declare function f({ [key]: v }: { p: number }): typeof v;
