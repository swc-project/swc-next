export default function f(value: string): string;
export default function f(value: number): number;
export default function f(value: string | number) {
  return value;
}
