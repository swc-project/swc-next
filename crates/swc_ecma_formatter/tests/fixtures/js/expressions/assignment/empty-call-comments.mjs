result = source.create(
  /* empty frame (with punctuation) remains part of this call */
).value;

result = source.invoke?.(
  // The empty optional frame retains its own comment packet.
);

result = invoke(
  /* A value-bearing frame retains its ordinary argument layout. */
  value
);
