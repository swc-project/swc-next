import { utf8Slice } from "./utf8.js";

const severities = ["error", "warning", "hint", "info"];

// Offsets are relative to this view, including when it starts within a buffer.
// Parser results retain an undefined help field; transform results omit it.
export function decodeDiagnostics(buffer, view, offset, count, omitAbsentHelp = false) {
  const result = new Array(count);
  if (count === 0) return result;
  const readU32 = () => {
    const value = view.getUint32(offset, true);
    offset += 4;
    return value;
  };
  const readString = () => {
    const length = readU32();
    const value = utf8Slice.call(buffer, offset, offset + length);
    offset += length;
    return value;
  };
  for (let index = 0; index < count; index++) {
    const severityValue = buffer[offset++];
    const severity = severities[severityValue];
    if (severity === undefined) {
      throw new RangeError(`Unknown diagnostic severity ${severityValue}`);
    }
    const start = readU32();
    const end = readU32();
    const message = readString();
    const help = buffer[offset++] === 0 ? undefined : readString();
    const labels = new Array(readU32());
    for (let label = 0; label < labels.length; label++) {
      labels[label] = { start: readU32(), end: readU32(), message: readString() };
    }
    result[index] = omitAbsentHelp && help === undefined
      ? { severity, message, start, end, labels }
      : { severity, message, start, end, help, labels };
  }
  return result;
}
