class Child extends (/* superclass */ new Base()) {}
class Ignored extends /* prettier-ignore */ (new   Base(  )) {}
new (/* callee */ class {})();
