// Adapted from SWC helpers. See THIRD_PARTY_NOTICES.
function _type_of(obj) {
    return obj && typeof Symbol !== "undefined" && obj.constructor === Symbol ? "symbol" : typeof obj;
}
export { _type_of as _ };
