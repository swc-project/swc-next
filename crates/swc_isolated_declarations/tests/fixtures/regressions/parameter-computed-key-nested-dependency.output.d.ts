declare const key: "prop";
export type F = ([{ nested: { [true ? key : key]: value } }]: [{
  nested: {
    prop: number;
  };
}]) => typeof value;
export {};
