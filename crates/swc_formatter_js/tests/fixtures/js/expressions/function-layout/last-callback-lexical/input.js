function outer(value) { return consume("lexical", next => { return [this.marker, arguments[0], next]; }); }
