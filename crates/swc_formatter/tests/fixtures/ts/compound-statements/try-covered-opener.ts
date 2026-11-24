try // @ts-expect-error
{ const value: string = 1;
  consume(value);
} catch {}
declare function consume(value: string): void;
