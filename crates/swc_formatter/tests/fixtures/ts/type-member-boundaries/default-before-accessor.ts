interface I {
  default;
  get x(): number;
}

type T = {
  default;
  set x(v: number);
};

interface Method {
  default;
  get(): number;
}

interface Comment {
  default;
  get // key
  x(): number;
}
