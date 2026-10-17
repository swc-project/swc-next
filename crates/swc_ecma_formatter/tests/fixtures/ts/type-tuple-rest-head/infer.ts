type Tail<T extends unknown[]> = T extends [...( /* head */ infer R )] ? R : never;
