export default interface I<T> {
  extra: T;
}
export default class C<T> {
  value!: T;
}
const I = 1;
const value = I;
