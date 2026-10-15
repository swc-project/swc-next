#include <node_api.h>
#include <valgrind/callgrind.h>
#include <valgrind/valgrind.h>

// This addon belongs to the benchmark harness, never to a published package.
static napi_value measure(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value callback, receiver, result;
  napi_valuetype type;
  if (napi_get_cb_info(env, info, &argc, &callback, NULL, NULL) != napi_ok ||
      argc != 1 || napi_typeof(env, callback, &type) != napi_ok || type != napi_function) {
    napi_throw_type_error(env, NULL, "measure requires a callback");
    return NULL;
  }
  if (!RUNNING_ON_VALGRIND) {
    napi_throw_error(env, NULL, "measurement requires Valgrind");
    return NULL;
  }
  napi_get_undefined(env, &receiver);
  CALLGRIND_START_INSTRUMENTATION;
  CALLGRIND_ZERO_STATS;
  CALLGRIND_TOGGLE_COLLECT;
  napi_status status = napi_call_function(env, receiver, callback, 0, NULL, &result);
  CALLGRIND_TOGGLE_COLLECT;
  CALLGRIND_STOP_INSTRUMENTATION;
  if (status != napi_ok) {
    bool pending = false;
    napi_is_exception_pending(env, &pending);
    if (!pending) napi_throw_error(env, NULL, "benchmark callback failed");
    return NULL;
  }
  return result;
}

NAPI_MODULE_INIT() {
  napi_value function;
  if (napi_create_function(env, "measure", NAPI_AUTO_LENGTH, measure, NULL, &function) != napi_ok ||
      napi_set_named_property(env, exports, "measure", function) != napi_ok) {
    napi_throw_error(env, NULL, "cannot initialize benchmark helper");
    return NULL;
  }
  return exports;
}
