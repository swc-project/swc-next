type Narrow<in T extends string | number = string, out U = T> = [T, U];
