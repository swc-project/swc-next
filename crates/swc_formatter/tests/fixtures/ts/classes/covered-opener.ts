class Example // @ts-expect-error
{ value: string = 1;
  read() { return this.value; }
}
