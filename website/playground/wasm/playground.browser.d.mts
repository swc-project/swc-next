export default function init(): Promise<void>;

/** Options are a JSON string decoded by serde in the playground's Rust crate. */
export function run(source: string, options: string): Uint8Array;
