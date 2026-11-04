result = Factory.first<number>
// prettier-ignore
({ one :1, two:2 }, (entry) => { const next = entry + 1; return next; }).second<string>(other);