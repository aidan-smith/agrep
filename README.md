# agrep

- Can parse literals, repetitions, and basic classes (ordinaries and ranges)

## TODO for AST Parsing

- [x] Unicode literals e.g. `'abc'`
- [ ] Escaping special characters with `\` e.g. `'\n'`
- [x] Repetitions with `'*'`, `'+'`, `'?'`, `{3}`, `{1,}`, `{1,3}`
- [x] Anchors e.g. `^foobar$`
- [x] Ordinary and range character classes e.g. `[A-z][0-9][!?]`
- [ ] Groups and alternations e.g. `foo(baz|bar)`
