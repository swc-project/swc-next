// Instance and static accessors with the same key are distinct properties.
// Their types must not be used to infer each other.
export declare class InstanceGetterStaticSetter {
  get value();
  static set value(value: string);
}
export declare class StaticGetterInstanceSetter {
  static get value();
  set value(value: number);
}
export declare class InstanceGetterStaticUntypedSetter {
  get value(): string;
  static set value(value);
}
export declare class StaticGetterInstanceUntypedSetter {
  static get value(): number;
  set value(value);
}
