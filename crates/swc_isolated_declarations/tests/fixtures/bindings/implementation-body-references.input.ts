import { Used, Unused } from "pkg";

type Private = Used;
const hidden = 1;

export function api(value: Private): void {
    console.log(hidden, Unused);
}
