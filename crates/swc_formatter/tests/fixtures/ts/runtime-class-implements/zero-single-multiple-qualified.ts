interface I { x: number; }
namespace N { export interface J<T> { x: T; } }
class Zero { x = 7; }
class One implements I { x = 7; }
class Many implements I, N.J<number> { x = 7; }
