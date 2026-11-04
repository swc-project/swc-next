declare const fn: number;
function f(){
// @ts-expect-error
return (
fn
)();
}
