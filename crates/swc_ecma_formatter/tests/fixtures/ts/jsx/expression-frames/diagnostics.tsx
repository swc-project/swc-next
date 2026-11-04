async function view(){return await (
// @ts-expect-error
<Panel first={firstArgument} second={secondArgument}/>);}
(first(), (
// @ts-expect-error
<Panel first={firstArgument} second={secondArgument}/>));
<Panel {...(
// @ts-expect-error
first || second)} />;
