class Base<T> { value = 7; }
interface I { value: number; }
abstract class A<T = number> extends Base<T> implements I {}
class B extends A<number> {}
