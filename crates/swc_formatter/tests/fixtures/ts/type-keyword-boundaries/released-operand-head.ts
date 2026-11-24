type Bound<T extends (// constraint
Target)> = T;

type Inferred = Input extends infer T extends (// inferred constraint
Target) ? T : never;

const cast = value as (// assertion
Target);

const checked = value satisfies (// consistency
Target);
