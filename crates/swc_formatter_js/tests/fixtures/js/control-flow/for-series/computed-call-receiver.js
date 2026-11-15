var let = [{ value: 29, read() { return this.value; } }];
let once = 0;
for ((let)[0].read(), once = 0; once < 1; once++) {
  consume(let[0].read());
}
