type Picked<T> = T extends string ? T extends "x" ? 1 : 2 : T extends number ? 3 : 4;
