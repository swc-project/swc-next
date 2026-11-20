export default class<T> {
  value!: T;
}
export default interface I<T> {
  extra: T;
}
const ctor = I;
