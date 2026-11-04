const check = () /* prettier-ignore */ : { value :number } => {
  // @ts-expect-error
  const value: number = "bad";
  return { value };
};