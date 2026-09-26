export declare namespace X.Y.Z {
	export {};
}
export declare namespace A.B.C {
	const value = 1;
	function foo(): void;
}
export declare namespace Deep.Nested.Namespace.Structure {
	interface Config {
		value: string;
	}
}

type Value = { value: number };
declare namespace Retained.Inner {
    const value: Value;
}
export type Public = typeof Retained.Inner;
export {};
