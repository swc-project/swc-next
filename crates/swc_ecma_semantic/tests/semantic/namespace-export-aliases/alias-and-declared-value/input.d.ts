declare namespace N {
  export const x: number;
}
declare namespace N {
  const y: string;
  export { y as x };
}
