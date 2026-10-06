let target = 1;
(
// @ts-expect-error
target as string
) = "text";
