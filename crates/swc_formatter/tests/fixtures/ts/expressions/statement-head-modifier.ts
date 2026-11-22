(static) as T;
(readonly) satisfies T;
(accessor) as const;
((public) as T) as U;
(private) as T;
if (ok) {
  (protected) satisfies T;
}
if (ok) (static) as T;
for ((static) as T; ok; step()) {}
