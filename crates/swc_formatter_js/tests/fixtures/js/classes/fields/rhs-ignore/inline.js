const configuration = { branch: { value: 3 } };
class RecordValue {
  intentionallyLongFieldName   = /* prettier-ignore */ configuration . branch . value;
  other = { untouched :  2 };
}
