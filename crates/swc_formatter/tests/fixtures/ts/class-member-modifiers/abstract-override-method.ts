abstract class Base { protected abstract value(): number; } class C extends Base { protected override value(): number { return 7; } read() { return this.value(); } }
