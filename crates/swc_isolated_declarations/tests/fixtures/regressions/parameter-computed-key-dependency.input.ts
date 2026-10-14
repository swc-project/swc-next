const key: "prop" = "prop";
export type F = ({ [key]: value }: { prop: number }) => typeof value;
