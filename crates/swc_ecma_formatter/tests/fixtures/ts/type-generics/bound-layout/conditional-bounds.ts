declare function select<T extends (Source extends Target ? Left : Right) = Source extends Target ? Left : Right>(): T;
