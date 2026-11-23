type Bound<T extends (// constraint
Target)> = T;

type Inferred = Input extends infer T extends (// inferred constraint
Target) ? T : never;
