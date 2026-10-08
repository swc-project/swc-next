function receiverAndTail(
  // prettier-ignore
  this:   { value:number },
  value?: number,
  ...rest: number[]
): number { return this.value + (value ?? rest.length); }
