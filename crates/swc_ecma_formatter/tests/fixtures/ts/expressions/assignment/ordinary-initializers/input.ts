function buildSomethingMeaningful(first: number, second: number): number { return first + second; }
const veryLongInitializedVariableName: number = buildSomethingMeaningful(1, 2);
function withDefault(veryLongDefaultParameterName: number = buildSomethingMeaningful(1, 2)): number { return veryLongDefaultParameterName; }
class Example { veryLongInitializedFieldName: number = buildSomethingMeaningful(1, 2); }
enum Value { veryLongComputedEnumMemberName = buildSomethingMeaningful(1, 2), literal = 1 }
