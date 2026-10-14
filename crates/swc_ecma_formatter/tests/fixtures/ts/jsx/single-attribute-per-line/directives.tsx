declare namespace JSX { interface IntrinsicElements { panel: { first?: boolean; second?: boolean; children?: unknown }; } }
// @ts-expect-error
const sameLine=<panel invalid first/>;
const attribute=<panel first // @ts-expect-error
invalid second/>;
const child=<panel first second>{/* @ts-expect-error */ missingValue}</panel>;
