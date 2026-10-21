import type { ConformanceKind, ConformanceResult, Count } from "../conformance-report";

declare const CONFORMANCE_REPORT: ConformanceResult[];
const number = (value: number) => value.toLocaleString("en-US");
const suiteNames: Record<string, string> = { test262: "Test262", typescript: "TypeScript", babel: "Babel", swc: "SWC", swc_resolver: "SWC resolver" };
const suiteOrder = ["test262", "typescript", "babel", "swc", "swc_resolver"];

function PassCount({ passed, total }: Count) {
  if (total === 0) return <span aria-label="No cases">—</span>;
  return <span className="conformance-count"><span>{number(passed)} / {number(total)}</span><span>{(passed / total * 100).toFixed(2)}%</span></span>;
}

export function ConformanceTable({ kind }: { kind: ConformanceKind }) {
  const results = CONFORMANCE_REPORT.filter((result) => result.kind === kind).sort((left, right) => suiteOrder.indexOf(left.suite) - suiteOrder.indexOf(right.suite));
  if (!results.length) throw new Error(`No conformance results for ${kind}`);
  const native = kind !== "estree" && kind !== "codec";
  const showNegative = results.some((result) => (result.negative?.total ?? 0) > 0);
  const showSkipped = results.some((result) => (result.skipped ?? 0) > 0);
  return <div className="conformance-table rp-scrollbar">
    <table>
      <thead><tr><th>Suite</th>
        {native ? <><th>Positive passed</th>{showNegative && <th>Negative passed</th>}</> : <>{kind === "estree" && <th>AST parsed</th>}<th>{kind === "estree" ? "AST matched" : "Codec passed"}</th></>}
        <th>{native ? "Failed" : "Not passed"}</th>{showSkipped && <th>Skipped</th>}
      </tr></thead>
      <tbody>{results.map((result) => <tr key={result.snapshot}>
        <th scope="row"><a href={`https://github.com/swc-project/swc-next/blob/main/tasks/testsuite/snapshots/${result.snapshot}`} target="_blank" rel="noreferrer">{suiteNames[result.suite] ?? result.suite}</a></th>
        {native ? <><td><PassCount {...result.positive!} /></td>{showNegative && <td><PassCount {...result.negative!} /></td>}</> : <>{result.parsed && <td><PassCount {...result.parsed} /></td>}<td><PassCount passed={result.passed} total={result.total} /></td></>}
        <td>{number(result.failed)}</td>{showSkipped && <td>{number(result.skipped ?? 0)}</td>}
      </tr>)}</tbody>
    </table>
  </div>;
}
