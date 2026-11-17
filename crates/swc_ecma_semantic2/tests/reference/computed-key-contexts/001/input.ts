interface T {}
const k = "p";
type F = ({ [(k as T) + k]: v }) => void;
