declare const React: any, Box: any;
declare function consume(value: number): any;
const view = <Box>{consume(
  // @ts-expect-error
  "wrong"
) /* suffix */ !}</Box>;
