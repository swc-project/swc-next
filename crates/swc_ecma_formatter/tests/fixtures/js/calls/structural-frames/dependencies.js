subscribe(() => { /* body */ work(); }, [firstDependency, secondDependency]);
subscribe(ref, () => { work(); }, [firstDependency, secondDependency]);
subscribe((() => { work(); }), ([firstDependency, secondDependency]));
subscribe((value) => { work(value); }, [firstDependency, secondDependency]);
subscribe(() => value, [firstDependency, secondDependency]);
subscribe(object.ref, () => { work(); }, [firstDependency, secondDependency]);
subscribe(() => { work(); }, /* boundary */ [firstDependency, secondDependency]);
subscribe((() => { work(); }) /* callback tail */, [firstDependency, secondDependency]);
