type Parameter = Input extends ((arg: (X extends Y ? Yes : No)) => Result) ? Yes : No;

type Constructor = Input extends (new <T = (X extends Y ? Yes : No)>() => Result) ? Yes : No;

type Inferred = Input extends [infer T extends ((this: (X extends Y ? Yes : No)) => Result)] ? T : never;
