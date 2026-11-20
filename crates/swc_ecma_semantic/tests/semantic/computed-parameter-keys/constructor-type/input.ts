const key = "p";
type F = new ({ [key]: v }: { p: number }) => object;
