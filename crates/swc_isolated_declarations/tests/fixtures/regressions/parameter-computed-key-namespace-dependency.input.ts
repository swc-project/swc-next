namespace N {
  export const key: "prop" = "prop";
}
export type F = ({ [N.key]: value }: { prop: number }) => typeof value;
