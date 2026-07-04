type RequireWrapper = (
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  exports: any,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  require: any,
  module: Module,
  __filename: string,
  __dirname: string
) => void;