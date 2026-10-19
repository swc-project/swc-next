declare const dec: any;
class C {
// @ts-expect-error
@dec value: number = "bad";
}
