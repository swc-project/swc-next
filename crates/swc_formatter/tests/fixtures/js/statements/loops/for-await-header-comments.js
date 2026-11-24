async function consume(values) {
  for /* Before await */ await /* Before opener */ (/* Binding */ const value of values) {
    use(value);
  }
}
