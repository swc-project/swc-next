class Counter {
  #value(
    this:   Counter,
    value: number
  ): number { return value; }
  read(): number {
    // @ts-expect-error
    this.#value("bad");
    return this.#value(3);
  }
}
