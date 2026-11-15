result = source.invoke<string>(
  /* The type argument and empty value frame have different owners. */
);

result = invoke<string>(
  /* This argument value prevents compact empty-frame classification. */
  value
);
