type X<T> = T extends A ? B : T extends { [K in keyof Ref<aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccccccccccc>]: Value<K> } ? C : D;
