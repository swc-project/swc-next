namespace N {
  export interface T {}
  export import B = N.T;
}
import A = N.B;
export { A };
