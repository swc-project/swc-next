class RecordType {
  "quoted-key" = 1;
  // @ts-expect-error
  field: string;
  // prettier-ignore
  untouched : string;
  method() {}
}

interface GetterMember { readonly /* prettier-ignore */ get; "quoted-key": number; }
interface SetterMember { readonly /* prettier-ignore */ set; "quoted-key": number; }
interface StaticMember { readonly /* prettier-ignore */ static; "quoted-key": number; }
interface PlainMembers { get; set; static; "quoted-key": number; }
