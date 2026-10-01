export declare class PrivateAccessors {
  private get getter();
  private set setter(value);
}
export declare function untypedDeclaration(this): void;
export declare const untypedExpression: (this) => void;
export declare class UntypedMethod {
  method(this): void;
}
export declare class UntypedSetter {
  set value(this, value: string);
}
export declare const untypedObjectMethod: {
  method(this): void;
};
