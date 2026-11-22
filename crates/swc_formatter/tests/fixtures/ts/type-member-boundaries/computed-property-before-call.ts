interface Computed {
  [k];
  (): void;
}

type Optional = { [k]?; <T>(): T };

interface Annotated {
  [k]: string;
  (): void;
}

interface Method {
  [get];
  m(): void;
}
