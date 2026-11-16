const build = () => (@decorate class Item { static value = 1; });
const ordinary = () => (class Item { static value = 2; });
const value = build().value + ordinary().value;
