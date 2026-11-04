abstract class RecordType { abstract plain(): void; abstract "quoted-key"(): void; method?(): void; get value(): number { return 1; } set value(next: number) {} }

class GenericConstructorMethod { "quoted-key"() {} static constructor<T>() {} }
class QuotedConstructorMethod { plain() {} static "constructor"<T>() {} }
class ConstructorDirective {
  "quoted-key"() {}
  // @ts-expect-error
  static constructor<T>() {}
}
abstract class BodylessConstructorMethod { abstract "quoted-key"(): void; static constructor<T>(): void; }
abstract class QuotedOptionalConstructorMethod { abstract plain(): void; static "constructor"?<T>(): void; }
class OrdinaryConstructor { "quoted-key"() {} constructor() {} }
abstract class OptionalConstructorMethod { abstract "quoted-key"(): void; static constructor?<T>(): void; }
