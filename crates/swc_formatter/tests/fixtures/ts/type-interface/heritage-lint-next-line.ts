interface A extends
// eslint-disable-next-line
LongNamespace.FirstBase<string>, OtherNamespace.SecondBase<number> {}

interface B extends
// eslint-disable-next-line
FirstBase, SecondBase,
ThirdBase <boolean> { y : boolean }
