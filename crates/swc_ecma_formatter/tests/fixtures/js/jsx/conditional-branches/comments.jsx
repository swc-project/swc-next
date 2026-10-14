flag ? /* head */ <Box><Item /><Item /></Box> : fallback;
flag ? <Box><Item /><Item /></Box> // tail
: fallback;
flag ? <Box><Item /><Item /></Box> : // nil
undefined;
flag ? (/* source head */ other ? value : <Box><Item /><Item /></Box>) : fallback;
flag ? /*#__PURE__*/ factory() : <Box><Item /><Item /></Box>;
flag ? (/*#__PURE__*/ factory()) : <Box><Item /><Item /></Box>;
flag ? // prettier-ignore
factory(  first,second ) : <Box><Item /><Item /></Box>;
