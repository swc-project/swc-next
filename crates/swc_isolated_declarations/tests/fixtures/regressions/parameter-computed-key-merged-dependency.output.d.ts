declare const key: "prop";
export type F = ({ [key]: value }: {
  prop: number;
}) => typeof value;
export {};
