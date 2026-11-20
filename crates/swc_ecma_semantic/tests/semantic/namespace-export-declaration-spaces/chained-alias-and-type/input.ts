namespace A {
  export interface T {}
}
import B = A;
import X = B.T;
type X = number;
