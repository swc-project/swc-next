declare namespace JSX {
  interface Element {}
  interface IntrinsicElements {
    div: {};
    "A-B": {};
  }
}
declare const _Button: () => JSX.Element;
declare const $Button: () => JSX.Element;
declare const Élement: () => JSX.Element;
const views = [<_Button />, <$Button />, <Élement />, <div />, <A-B />];
