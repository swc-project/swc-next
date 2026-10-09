const key = "method"; class C { value = 1; [key](input: number): number; [key](input: number): number { return input + this.value; } }
