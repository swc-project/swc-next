const a1=<Box>{ /* head */ value /* tail */ }{value // last
}{/* one */ /* two */}{// empty line
}</Box>;
const x1=<Box { /* spread */ ...props }>{ /* child */ ...children }</Box>;

const object_attribute=<Box data={{firstVeryLongKey: "first", secondVeryLongKey: "second", thirdVeryLongKey: "third"}}/>;
const array_attribute=<Box data={["firstVeryLongValue", "secondVeryLongValue", "thirdVeryLongValue"]}/>;
const object_array_children=<Box>{{firstVeryLongKey: "first", secondVeryLongKey: "second"}}{["firstVeryLongValue", "secondVeryLongValue"]}</Box>;
const wrapped_collections=<Box data={({firstVeryLongKey: "first", secondVeryLongKey: "second"})} list={((["firstVeryLongValue", "secondVeryLongValue"]))}/>;
const commented_collections=<Box data={/* head object */ {firstVeryLongKey: "first", secondVeryLongKey: "second"} /* tail object */}>{({/* inner object */ firstVeryLongKey: "first"})}{["firstVeryLongValue" /* inner array */]}</Box>;
const ignored_collections=<Box data={/* prettier-ignore */ { firstVeryLongKey:  "first",secondVeryLongKey:"second" }} list={/* prettier-ignore */ [ "firstVeryLongValue",  "secondVeryLongValue" ]}/>;

const ordinary_calls=<Box data={String.prototype.concat.call("firstVeryLongValue", "secondVeryLongValue", "thirdVeryLongValue")}>{Math.max(123456789, 234567891, 345678912)}</Box>;
const ordinary_functions=<Box marker="receiver" data={(function(firstVeryLongParameter, secondVeryLongParameter){ return <Box data={this.marker}>{String(firstVeryLongParameter).concat(secondVeryLongParameter)}</Box>; })}>{function(childValue){ return <Box>{childValue}</Box>; }}</Box>;
ordinary_functions.props.data("first", "second");
ordinary_functions.children[0]("child");
const guarded_calls_functions=<Box data={/* prettier-ignore */ String.prototype.concat.call( "first",  "second")}>{ /* owner */ function(childValue){ return childValue; } /* tail */ }</Box>;

const logical_attribute=<Box data={value && props && children && value && props && children}/>;
const logical_child=<Box>{value && props && children && value && props && children}</Box>;
const logical_spread=<Box {...(value && props && children && props)}>{...(value && children)}</Box>;
const logical_comments=<Box data={/* head logical */ (value && props && children) /* tail logical */}/>;
const logical_ignored=<Box data={/* prettier-ignore */ value  &&  props && children}/>;
const logical_active=<Box data={value && /* @__PURE__ */ Object.assign({}, props, {marker: "logical active"})}/>;

const jsx_arrow_owners=<Box data={value => <Box>{value}</Box>} mapped={items.map(value => <Box>{value}</Box>)}>{value => next => <Box>{value + next}</Box>}{value => "firstVeryLongValuefirstVeryLongValue" + value + "secondVeryLongValuesecondVeryLongValue"}</Box>;
const jsx_arrow_source=<Box data={/* prettier-ignore */ value => ( <Box  data = { value }>x y</Box> )}>{/* head arrow */ value => <Box data={value}>{value}</Box> /* tail arrow */}</Box>;
