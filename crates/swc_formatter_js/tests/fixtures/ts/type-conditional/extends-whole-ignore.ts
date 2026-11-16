type Outcome<T> = T extends
/* prettier-ignore */
( string  | number ) ? 1 : 2;
