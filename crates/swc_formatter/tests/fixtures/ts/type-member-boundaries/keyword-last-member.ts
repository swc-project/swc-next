type Flat = { get };
type Pair = { get; set };
type Mixed = { a: string; static };
type Broken = { aaaaaaaaaaaa: string; get };
type Lead = { set; a: string };
interface I { get }
interface J { a: string; set }
