function make(argument: string) {
  return (value: any, context: any) => value;
}
// @ts-expect-error
const complete = @make(1) class Item { static value: string = 1; };
const enclosed = (
  // @ts-expect-error
  @make(1) class Item { static value: string = 1; }).missing();
