export {};
declare namespace JSX {
  interface Element {}
  interface IntrinsicElements {
    div: {};
    "A-B": {};
  }
}
declare const ui: { Button: () => JSX.Element };
const view = <ui.Button />;
