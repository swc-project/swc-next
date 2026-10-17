((/* first */ <Panel first={firstArgument} second={secondArgument}/> // after first
), next());
(first(), (/* middle */ <Panel first={firstArgument} second={secondArgument}/> /* after middle */), last());
(first(), (/* last */ <Panel first={firstArgument} second={secondArgument}/> // after last
));
