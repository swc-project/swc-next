const direct = flag ? <Box><Item /><Item /></Box> : null;
const alternate = flag ? value : other ? <Box><Item /><Item /></Box> : fallback;
const consequent = flag ? other ? <Box><Item /><Item /></Box> : value : fallback;
const nil = flag ? null : other ? undefined : <><Item /><Item /></>;
const receiver = (flag ? <Box><Item /><Item /></Box> : null).value;
(flag ? <Box><Item /><Item /></Box> : null)();
const plain = first && second && third ? value : fallback;
