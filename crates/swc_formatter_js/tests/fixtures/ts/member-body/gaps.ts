interface Calls { first(): void;

second(): void; // trailing

third(): void;
// leading

fourth(): void; }
type Properties = { first: number;

second: string;

// leading
third: boolean; };
interface InlinePacket { first(): void; /* leading */ second(): void; }
type InlinePacket = { firstLongProperty: number; /* leading */ secondLongProperty: string };
