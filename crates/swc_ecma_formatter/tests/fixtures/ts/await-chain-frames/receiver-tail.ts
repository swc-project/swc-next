async function run() {
  await (await source.load(request) /* receiver */).select(options).finish();
}
