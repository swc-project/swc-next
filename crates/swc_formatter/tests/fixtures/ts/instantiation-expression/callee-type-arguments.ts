const value = (apply<Inner>)<Outer>(argument, anotherArgument);
const nested = ((apply<First>)<Second>)(value);
const nestedOptional = ((apply?.callback<First>)<Second>)(value);
const optional = (apply<Inner>)?.<Outer>(value);

new (Constructor<Inner>)<Outer>();

class Derived extends (Base<Inner>)<Outer> {}

class Nested extends ((Base<First>)<Second>) {}
