// @ts-expect-error
if (firstCondition) if (secondCondition) firstMissing(); else secondMissing(); else thirdMissing();
