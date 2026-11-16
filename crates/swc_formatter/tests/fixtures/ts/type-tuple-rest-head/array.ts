type ArrayTail<T extends unknown[]> = T extends [...(/* element */ infer R)[]] ? R : never;
