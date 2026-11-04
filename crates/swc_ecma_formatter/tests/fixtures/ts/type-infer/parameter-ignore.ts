type Outcome<T> = T extends infer /* prettier-ignore */ R   extends   (string | number) ? R : never;
