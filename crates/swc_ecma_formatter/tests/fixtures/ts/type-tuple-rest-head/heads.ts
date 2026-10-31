type Tail<T extends unknown[]> = T extends [.../* rest */ (/* outer */ (/* inner */ infer R))[]] ? R : never;
