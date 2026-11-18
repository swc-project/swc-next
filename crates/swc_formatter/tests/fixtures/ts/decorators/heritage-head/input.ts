// @ts-expect-error
class C extends (@missingDecorator class {}) {}
outsideMissing;
