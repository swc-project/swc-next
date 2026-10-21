declare const use: (x: any)=>void; declare const fn: number;
// @ts-expect-error
use((
fn
)());
