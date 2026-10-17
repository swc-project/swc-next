declare namespace JSX { interface IntrinsicElements { div: {valid?: string, children?: unknown}; } }
const element = <div {...{} /*
 // @ts-ignore */ /* prettier-ignore */}
 invalidProp="invalid" />;
