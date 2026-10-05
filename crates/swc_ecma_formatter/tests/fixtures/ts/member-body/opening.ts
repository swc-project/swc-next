type Inline = { /* inline */ value: number };
type Line = { // line
value: number };
type Block = { /* block */
value: number };
interface Leading {
/* leading */ value: number;
}
type Callback = (value: { // nested
first: number;
second: string;
}) => void;
