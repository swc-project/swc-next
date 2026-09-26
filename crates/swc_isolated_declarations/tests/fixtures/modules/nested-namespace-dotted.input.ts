// Dotted namespace syntax should preserve all names correctly
export namespace X.Y.Z {}

// Nested dotted namespaces with content
export namespace A.B.C {
    export const value = 1;
    export function foo(): void {}
}

// Deeply nested
export namespace Deep.Nested.Namespace.Structure {
    export interface Config {
        value: string;
    }
}

import type { Unused } from "unused";
declare function make(): Unused;
namespace UnusedNamespace.Inner {
    export const value = make();
}

type Value = { value: number };
namespace Retained.Inner {
    export const value: Value = { value: 1 };
}
export type Public = typeof Retained.Inner;
