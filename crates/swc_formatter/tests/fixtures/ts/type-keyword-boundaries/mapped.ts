type Constrained = { [\u004b // prettier-ignore
 in(( Input ))]: K };

type Remapped = { [K in Input  [  ]/* prettier-ignore */as(Name)]: K };
