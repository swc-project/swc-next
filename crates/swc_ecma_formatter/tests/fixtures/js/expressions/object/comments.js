({ /*empty*/ });
({ //empty
});
({ a /*key*/ : /*value*/ b, c //tail
});
({ /*lead*/ a: 1, /*next*/ b: 2 /*end*/ });
({ [/*key*/ a /*end*/]: b, .../*spread*/ c });
({ a: 1,

b: 2,
// dangling
});
