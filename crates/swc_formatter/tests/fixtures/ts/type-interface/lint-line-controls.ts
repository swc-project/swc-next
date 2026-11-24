interface A // ordinary header comment
{ x: number }

interface B extends
// ordinary heritage comment
LongNamespace.FirstBase<string>, OtherNamespace.SecondBase<number> {}

interface C {
  x : number // eslint-disable-line
  y : string
}
