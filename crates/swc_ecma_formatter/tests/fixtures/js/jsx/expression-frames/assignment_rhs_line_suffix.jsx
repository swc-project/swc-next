const comment // marker
= <Foo>{/* existing */}</Foo>;

const element // marker
= <Foo bar={baz} {...props}>word<span /></Foo>;

const fragment // marker
= <>word<Foo /></>;
