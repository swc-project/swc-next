export interface CodegenOptions {
  asciiOnly?: boolean;
  minify?: boolean;
}

export interface CodegenOutput {
  code: string;
}

export declare function printSync(
  program: any,
  options?: CodegenOptions | undefined | null,
): CodegenOutput;
