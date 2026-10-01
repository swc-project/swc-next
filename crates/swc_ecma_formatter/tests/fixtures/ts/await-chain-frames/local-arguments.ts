async function run() {
  await (await source.load(request)).select({ first: firstValue, second: secondValue, third: thirdValue }).finish();
}
