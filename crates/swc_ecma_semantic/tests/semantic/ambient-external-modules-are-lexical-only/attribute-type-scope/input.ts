type Attribute = "text";
declare module "*.txt" with { type: Attribute } {
  type Attribute = "inner";
}
