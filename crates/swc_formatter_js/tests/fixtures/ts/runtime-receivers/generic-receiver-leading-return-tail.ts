function genericReceiver<T>
// prettier-ignore
(this:   { value:T }, value: T) // prettier-ignore
:   T { return value; }
