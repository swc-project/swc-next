function createContentRange(type: string, size: number, range?: Range): string {
  return `${type} ${range ? `${range.start}-${range.end}` : '*'}:${size}`.replace(
    ':',
    '/',
  );
}
