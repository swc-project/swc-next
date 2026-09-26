export async function named(value: string): Promise<void>;
export async function named(value: unknown): Promise<void> {}

export default async function defaultFunction(value: string): Promise<void>;
export default async function defaultFunction(value: unknown): Promise<void> {}

export namespace N {
  export async function nested(value: string): Promise<void>;
  export async function nested(value: unknown): Promise<void> {}
}
