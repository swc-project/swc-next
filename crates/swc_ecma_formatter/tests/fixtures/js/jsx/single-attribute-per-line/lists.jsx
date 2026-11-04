const empty=<Box/>;
const single=<Box active/>;
const singleSpread=<Box {...properties}/>;
const singleText=<Box title="a long string attribute stays together"/>;
const attributes=<Box active focus value={first+second}/>;
const spread=<Box {...properties} active {...other}/>;
const multiline=<Box title="first
  second" active/>;
