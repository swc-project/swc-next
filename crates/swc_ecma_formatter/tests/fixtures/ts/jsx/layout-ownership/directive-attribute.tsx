declare function consumeString(value: string): void;
const view=<Panel value={(value: number)=>// @ts-ignore
consumeString(value)}/>;
