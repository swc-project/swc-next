const key = "p";
function f({ [key]: v }: { p: number }): typeof v;
