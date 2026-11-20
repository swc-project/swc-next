namespace N {
  export interface T {}
}
import B = N;
import A = B;
export type Result = A.T;
