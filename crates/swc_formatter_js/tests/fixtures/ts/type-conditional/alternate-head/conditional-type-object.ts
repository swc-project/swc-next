type X<T> = T extends A ? B : T extends { aaaaaaaaaaaaaa: string; bbbbbbbbbbbbb: number; cccccccccccccc: boolean; ddddddddd: string } ? C : D;
