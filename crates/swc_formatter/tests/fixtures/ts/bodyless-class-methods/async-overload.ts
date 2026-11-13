class C {
  public async method<T>(value: T): Promise<T>;
  public async method(value: unknown) { return value; }
}
