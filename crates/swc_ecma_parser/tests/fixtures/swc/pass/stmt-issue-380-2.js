class Foo {
                componentDidMount() {
                    const filePath = '../foo/bar'
                    import(filePath).then(bar => {})
                }
            }