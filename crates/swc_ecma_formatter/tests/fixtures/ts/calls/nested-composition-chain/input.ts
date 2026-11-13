const f = () => {
  const schema = z.string().check(
    z.refine((val) => val.length > 1),
    z.refine((val) => val.length > 2, { abort: true }),
  );
};
