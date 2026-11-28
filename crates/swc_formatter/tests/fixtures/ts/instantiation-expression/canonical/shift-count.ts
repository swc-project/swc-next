async function* shifts(bits: number) {
  const direct = bits << count<number>;
  const awaited = direct << await count<number>;
  return awaited << ((/* prettier-ignore */ yield count  < number >));
}
