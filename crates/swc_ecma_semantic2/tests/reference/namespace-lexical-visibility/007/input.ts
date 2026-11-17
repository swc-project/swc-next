declare namespace N {
  namespace Target {
    interface Type {}
  }
  export import PublicAlias = Target;
  import PrivateAlias = Target;
}
declare namespace N {
  interface UsesPublic extends PublicAlias.Type {}
  type UsesPrivate = PrivateAlias;
}
