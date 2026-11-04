namespace N { export interface I<T> { x: T; } }
class Header<T> // prettier-ignore
implements N.I<number> { x = 7; }
class Eol implements N . I // prettier-ignore
<  number  > { x = 7; }
