async function run() {
  await (
    // @ts-ignore
    await source.load(request)
  ).select(options).finish();
}
