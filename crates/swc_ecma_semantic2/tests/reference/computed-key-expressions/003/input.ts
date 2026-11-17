const key = (v: string) => v;
type F = ({ [key(key)]: v }: object) => typeof v;
