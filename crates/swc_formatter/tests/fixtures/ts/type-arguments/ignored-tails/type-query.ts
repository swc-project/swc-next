type A = typeof fn<F< 'x' > /* prettier-ignore */>;

type B = typeof fn<( <T>() => void ) /* prettier-ignore */>;

type C = typeof fn< <T>() => void /* prettier-ignore */>;

type D = typeof fn<string, 'x' /* prettier-ignore */>;
