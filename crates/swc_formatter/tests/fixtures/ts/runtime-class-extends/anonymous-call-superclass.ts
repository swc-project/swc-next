class Base<T> {}
function choose<T>() { return Base; }
const Derived = class extends (choose<string>())<number> {};
