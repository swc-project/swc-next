new Observable<number>((subscriber) => subscriber.error(failure))[distinctUntilChanged]().subscribe({ error: (error) => errors.push(error) });
