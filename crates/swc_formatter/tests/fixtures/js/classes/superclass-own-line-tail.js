class Derived extends (Base
// superclass
) {}

const Example = class extends (getBase()
// superclass
) {
  method() {}
};

class Ignored extends (/* prettier-ignore */ pkg . Base
// superclass
) {}
