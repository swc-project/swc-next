use crate::expect_printed_ts;

#[test]
fn test_export_default() {
    expect_printed_ts(
        b"export default function(): void;",
        b"export default function(): void;\n",
    );
    expect_printed_ts(
        b"export default async function<T>(): Promise<T>;",
        b"export default async function<T>(): Promise<T>;\n",
    );
}
