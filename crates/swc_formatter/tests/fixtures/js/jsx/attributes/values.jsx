const a1=<Box title='a\b &quot;c&quot; &amp; d' other="&apos;x&apos; &quot;y&quot;" active data={value+1} {...props}/>;
const a2=<Box title="first
  second	line &nbsp;
last"/>;
const a3=<div title="a
b">first
 second</div>;
const a4=<Box title=
/* value */
"foo"/>;
const a5=<Box title=// value
"foo"/>;
