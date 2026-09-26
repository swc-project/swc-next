declare function make(): Disposable;
declare function makeAsync(): AsyncDisposable;
using resource: Disposable = make();
await using asyncResource: AsyncDisposable = makeAsync();
export type Resource = typeof resource;
export type AsyncResource = typeof asyncResource;
