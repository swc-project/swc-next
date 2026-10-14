declare namespace N {
  const key: "prop";
}
export type F = ({ [N.key]: value }: {
  prop: number;
}) => typeof value;
export {};
