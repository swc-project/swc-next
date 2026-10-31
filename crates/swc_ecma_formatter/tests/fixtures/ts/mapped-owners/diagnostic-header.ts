type M = {
 readonly
 // @ts-expect-error
 [K in Missing]: number;
};
