function isValidIndexKeyType(type) {
  return !!(type.flags & (4 /* String */ | 8 /* Number */ | 4096 /* ESSymbol */));
}
