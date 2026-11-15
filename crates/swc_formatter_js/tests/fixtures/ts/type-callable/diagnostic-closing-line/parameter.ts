type F= (x:number,
/* @ts-expect-error */ /* packet */
)=> MissingType;
type Outside= OutsideMissing;
