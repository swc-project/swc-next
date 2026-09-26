export declare namespace Foo {
	var a: number;
	function bar(): void;
}
export declare namespace Outer {
	namespace Inner {
		var x: number;
	}
}
export declare namespace Baz {
	var c: number;
}
declare global {
	interface GlobalTest {}
}
declare module "test-module" {
	var d: 4;
}
