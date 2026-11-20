declare module "foo" {
  export const x: number;
}
declare module "foo" {
  export const y: typeof x;
}
