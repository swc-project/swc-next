declare namespace N {
  const x: number;
}
declare namespace M {
  import A = N;
  const result: typeof A.x;
}
