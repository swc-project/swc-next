interface Box<T> {
  value: T;
}
const C = class Box<T> {
  value!: T;
};
