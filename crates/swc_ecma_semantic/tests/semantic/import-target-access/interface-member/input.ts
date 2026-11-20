namespace N {
  export const x = 1;
  export interface T {}
}
import A = N.T;
export type Result = A;
