type Key<T extends string>=`head${T}tail${T | "x"}`; type Choice<T>=`${T extends string ? "a" : "b"}`; type Empty=``; type Literal=`a\nb`; type Comments<T extends string>=`${/*head*/ T //tail
}`;
