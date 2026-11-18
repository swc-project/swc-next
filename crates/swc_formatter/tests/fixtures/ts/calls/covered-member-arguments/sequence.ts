function d(g) {
  const s = g._sources
    // @ts-ignore
    .reduce((acc, val, idx) => ((acc[val] = idx), acc), {});
}
