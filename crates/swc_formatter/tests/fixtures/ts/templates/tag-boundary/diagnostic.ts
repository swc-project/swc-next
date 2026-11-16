declare function tag(template: TemplateStringsArray, value: number): string;
const value = tag /* @ts-expect-error */
`${"invalid"}`;
