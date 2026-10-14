(condition ? first : second) ? <Box><Item /><Item /></Box> : null;
((condition ? <Item /> : null) ? first : second) ? value : fallback;
flag ? <Box><Item /><Item /></Box> : (condition ? first : second) ? value : fallback;
flag ? <Box><Item /><Item /></Box> : (condition ? <Item /> : null) ? value : fallback;
flag ? render(condition ? <Item /> : null) : fallback;
