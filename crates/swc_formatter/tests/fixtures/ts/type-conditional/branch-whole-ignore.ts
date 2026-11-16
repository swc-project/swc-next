type Outcome<T> = T extends string ?
/* prettier-ignore */
( 1  | 2 ) : 3;
