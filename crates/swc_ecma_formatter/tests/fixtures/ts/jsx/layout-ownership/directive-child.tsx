declare function consumeString(value: string): void;
const view=<Panel>{(value: number)=>// @ts-expect-error
consumeString(value)}</Panel>;
