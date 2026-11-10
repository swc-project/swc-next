import { _ as _set_function_name } from "./_set_function_name.js";

// Run before static fields, after methods have been installed. A static method
// or accessor called `name` takes precedence over the constructor's own name.
function _set_class_name(cls, name) {
  var descriptor = Object.getOwnPropertyDescriptor(cls, "name");
  if (descriptor && "value" in descriptor && !descriptor.writable) {
    _set_function_name(cls, name);
  }
  return cls;
}
export { _set_class_name as _ };
