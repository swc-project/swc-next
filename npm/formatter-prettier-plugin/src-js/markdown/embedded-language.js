export function resolveEmbeddedLanguage(language, plugins) {
  const name = language.toLowerCase();
  const registered = findLanguage(name, plugins);
  if (registered) return registered;
  if (name === "angular-ts") return findLanguage("typescript", plugins);
  if (
    name === "angular-html" &&
    plugins.some((plugin) => Object.hasOwn(plugin.parsers ?? {}, "angular"))
  ) {
    return { parser: "angular", filepath: "embedded.html" };
  }
}

function findLanguage(name, plugins) {
  if (!name) return;
  for (let index = plugins.length - 1; index >= 0; index--) {
    for (const candidate of plugins[index].languages ?? []) {
      const extension = candidate.extensions?.find(
        (ext) => ext.toLowerCase() === `.${name}`,
      );
      const filename = candidate.filenames?.find(
        (file) => file.toLowerCase() === name,
      );
      const matches =
        candidate.name.toLowerCase() === name ||
        candidate.aliases?.some((alias) => alias.toLowerCase() === name) ||
        extension ||
        filename;
      if (!matches || !candidate.parsers[0]) continue;
      const fileExtension = extension ?? candidate.extensions?.[0] ?? "";
      return {
        parser: candidate.parsers[0],
        filepath: filename ?? `embedded${fileExtension}`,
      };
    }
  }
}
