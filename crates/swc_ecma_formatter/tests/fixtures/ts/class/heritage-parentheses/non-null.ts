class ObjectBase extends ({}!) {}
class ConstructedBase extends (new Base()!) {}
class TaggedBase extends (make`base`!) {}
class IdentifierBase extends Base! {}
