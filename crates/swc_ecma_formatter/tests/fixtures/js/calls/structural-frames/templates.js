insertRule(`first line
second ${value} line`);
new Error(formatErrorMessage`first line
second line`);
import(String.raw`first line
second line`);
insertRule(
  `first line
second line`
);
insertRule(`single line`);
insertRule((`first line
second line`));
insertRule((
  `first line
second line`
));
insertRule((`first line
second line` /* tail */));
insertRule(/* lead */ `first line
second line`);
insertRule(`first line
second line`, option);
