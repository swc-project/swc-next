use string_interner::{StringInterner, backend::StringBackend};
use swc_core::atoms::wtf8::{Wtf8, Wtf8Buf};

use crate::{OptionalUtf8Ref, OptionalWtf8Ref, Utf8Ref, Wtf8Ref};

pub const UTF8_EMPTY: Utf8Ref = Utf8Ref::new(0);
pub const UTF8_ABSTRACT: Utf8Ref = Utf8Ref::new(1);
pub const UTF8_ACCESSOR: Utf8Ref = Utf8Ref::new(2);
pub const UTF8_ARGUMENTS: Utf8Ref = Utf8Ref::new(3);
pub const UTF8_AS: Utf8Ref = Utf8Ref::new(4);
pub const UTF8_ASYNC: Utf8Ref = Utf8Ref::new(5);
pub const UTF8_AWAIT: Utf8Ref = Utf8Ref::new(6);
pub const UTF8_BOOLEAN: Utf8Ref = Utf8Ref::new(7);
pub const UTF8_BREAK: Utf8Ref = Utf8Ref::new(8);
pub const UTF8_BYTE: Utf8Ref = Utf8Ref::new(9);
pub const UTF8_CASE: Utf8Ref = Utf8Ref::new(10);
pub const UTF8_CATCH: Utf8Ref = Utf8Ref::new(11);
pub const UTF8_CHAR: Utf8Ref = Utf8Ref::new(12);
pub const UTF8_CLASS: Utf8Ref = Utf8Ref::new(13);
pub const UTF8_CONST: Utf8Ref = Utf8Ref::new(14);
pub const UTF8_CONSTRUCTOR: Utf8Ref = Utf8Ref::new(15);
pub const UTF8_CONTINUE: Utf8Ref = Utf8Ref::new(16);
pub const UTF8_DEBUGGER: Utf8Ref = Utf8Ref::new(17);
pub const UTF8_DECLARE: Utf8Ref = Utf8Ref::new(18);
pub const UTF8_DEFAULT: Utf8Ref = Utf8Ref::new(19);
pub const UTF8_DELETE: Utf8Ref = Utf8Ref::new(20);
pub const UTF8_DO: Utf8Ref = Utf8Ref::new(21);
pub const UTF8_DOUBLE: Utf8Ref = Utf8Ref::new(22);
pub const UTF8_ELSE: Utf8Ref = Utf8Ref::new(23);
pub const UTF8_ENUM: Utf8Ref = Utf8Ref::new(24);
pub const UTF8_EVAL: Utf8Ref = Utf8Ref::new(25);
pub const UTF8_EXPORT: Utf8Ref = Utf8Ref::new(26);
pub const UTF8_EXTENDS: Utf8Ref = Utf8Ref::new(27);
pub const UTF8_FALSE: Utf8Ref = Utf8Ref::new(28);
pub const UTF8_FINAL: Utf8Ref = Utf8Ref::new(29);
pub const UTF8_FINALLY: Utf8Ref = Utf8Ref::new(30);
pub const UTF8_FLOAT: Utf8Ref = Utf8Ref::new(31);
pub const UTF8_FOR: Utf8Ref = Utf8Ref::new(32);
pub const UTF8_FUNCTION: Utf8Ref = Utf8Ref::new(33);
pub const UTF8_GET: Utf8Ref = Utf8Ref::new(34);
pub const UTF8_GOTO: Utf8Ref = Utf8Ref::new(35);
pub const UTF8_IF: Utf8Ref = Utf8Ref::new(36);
pub const UTF8_IMPLEMENTS: Utf8Ref = Utf8Ref::new(37);
pub const UTF8_IMPORT: Utf8Ref = Utf8Ref::new(38);
pub const UTF8_IN: Utf8Ref = Utf8Ref::new(39);
pub const UTF8_INSTANCEOF: Utf8Ref = Utf8Ref::new(40);
pub const UTF8_INT: Utf8Ref = Utf8Ref::new(41);
pub const UTF8_INTERFACE: Utf8Ref = Utf8Ref::new(42);
pub const UTF8_LET: Utf8Ref = Utf8Ref::new(43);
pub const UTF8_LONG: Utf8Ref = Utf8Ref::new(44);
pub const UTF8_NATIVE: Utf8Ref = Utf8Ref::new(45);
pub const UTF8_NEW: Utf8Ref = Utf8Ref::new(46);
pub const UTF8_NULL: Utf8Ref = Utf8Ref::new(47);
pub const UTF8_OVERRIDE: Utf8Ref = Utf8Ref::new(48);
pub const UTF8_PACKAGE: Utf8Ref = Utf8Ref::new(49);
pub const UTF8_PRIVATE: Utf8Ref = Utf8Ref::new(50);
pub const UTF8_PROTECTED: Utf8Ref = Utf8Ref::new(51);
pub const UTF8_PUBLIC: Utf8Ref = Utf8Ref::new(52);
pub const UTF8_READONLY: Utf8Ref = Utf8Ref::new(53);
pub const UTF8_RETURN: Utf8Ref = Utf8Ref::new(54);
pub const UTF8_SET: Utf8Ref = Utf8Ref::new(55);
pub const UTF8_SHORT: Utf8Ref = Utf8Ref::new(56);
pub const UTF8_STATIC: Utf8Ref = Utf8Ref::new(57);
pub const UTF8_SUPER: Utf8Ref = Utf8Ref::new(58);
pub const UTF8_SWITCH: Utf8Ref = Utf8Ref::new(59);
pub const UTF8_SYNCHRONIZED: Utf8Ref = Utf8Ref::new(60);
pub const UTF8_THIS: Utf8Ref = Utf8Ref::new(61);
pub const UTF8_THROW: Utf8Ref = Utf8Ref::new(62);
pub const UTF8_THROWS: Utf8Ref = Utf8Ref::new(63);
pub const UTF8_TRANSIENT: Utf8Ref = Utf8Ref::new(64);
pub const UTF8_TRUE: Utf8Ref = Utf8Ref::new(65);
pub const UTF8_TRY: Utf8Ref = Utf8Ref::new(66);
pub const UTF8_TYPE: Utf8Ref = Utf8Ref::new(67);
pub const UTF8_TYPEOF: Utf8Ref = Utf8Ref::new(68);
pub const UTF8_USING: Utf8Ref = Utf8Ref::new(69);
pub const UTF8_VAR: Utf8Ref = Utf8Ref::new(70);
pub const UTF8_VOID: Utf8Ref = Utf8Ref::new(71);
pub const UTF8_VOLATILE: Utf8Ref = Utf8Ref::new(72);
pub const UTF8_WHILE: Utf8Ref = Utf8Ref::new(73);
pub const UTF8_WITH: Utf8Ref = Utf8Ref::new(74);
pub const UTF8_YIELD: Utf8Ref = Utf8Ref::new(75);

/// A string allocator that can be used to allocate strings for the AST.
/// All the strings are stored in a single buffer to avoid memory fragmentation.
pub struct StringAllocator {
    allocated_utf8: StringInterner<StringBackend<Utf8Ref>>,
    allocated_wtf8: Wtf8Buf,
}

impl StringAllocator {
    /// Create a new string allocator with the given source length.
    /// The source length is used to pre-allocate memory for the string allocator.
    /// We assume that half length of the source code is the sum of utf8 identifier lengths.
    pub fn new(source_len: usize) -> Self {
        let mut allocated_utf8 = StringInterner::with_capacity(source_len / 2);
        allocated_utf8.get_or_intern("");
        allocated_utf8.get_or_intern("abstract");
        allocated_utf8.get_or_intern("accessor");
        allocated_utf8.get_or_intern("arguments");
        allocated_utf8.get_or_intern("as");
        allocated_utf8.get_or_intern("async");
        allocated_utf8.get_or_intern("await");
        allocated_utf8.get_or_intern("boolean");
        allocated_utf8.get_or_intern("break");
        allocated_utf8.get_or_intern("byte");
        allocated_utf8.get_or_intern("case");
        allocated_utf8.get_or_intern("catch");
        allocated_utf8.get_or_intern("char");
        allocated_utf8.get_or_intern("class");
        allocated_utf8.get_or_intern("const");
        allocated_utf8.get_or_intern("constructor");
        allocated_utf8.get_or_intern("continue");
        allocated_utf8.get_or_intern("debugger");
        allocated_utf8.get_or_intern("declare");
        allocated_utf8.get_or_intern("default");
        allocated_utf8.get_or_intern("delete");
        allocated_utf8.get_or_intern("do");
        allocated_utf8.get_or_intern("double");
        allocated_utf8.get_or_intern("else");
        allocated_utf8.get_or_intern("enum");
        allocated_utf8.get_or_intern("eval");
        allocated_utf8.get_or_intern("export");
        allocated_utf8.get_or_intern("extends");
        allocated_utf8.get_or_intern("false");
        allocated_utf8.get_or_intern("final");
        allocated_utf8.get_or_intern("finally");
        allocated_utf8.get_or_intern("float");
        allocated_utf8.get_or_intern("for");
        allocated_utf8.get_or_intern("function");
        allocated_utf8.get_or_intern("get");
        allocated_utf8.get_or_intern("goto");
        allocated_utf8.get_or_intern("if");
        allocated_utf8.get_or_intern("implements");
        allocated_utf8.get_or_intern("import");
        allocated_utf8.get_or_intern("in");
        allocated_utf8.get_or_intern("instanceof");
        allocated_utf8.get_or_intern("int");
        allocated_utf8.get_or_intern("interface");
        allocated_utf8.get_or_intern("let");
        allocated_utf8.get_or_intern("long");
        allocated_utf8.get_or_intern("native");
        allocated_utf8.get_or_intern("new");
        allocated_utf8.get_or_intern("null");
        allocated_utf8.get_or_intern("override");
        allocated_utf8.get_or_intern("package");
        allocated_utf8.get_or_intern("private");
        allocated_utf8.get_or_intern("protected");
        allocated_utf8.get_or_intern("public");
        allocated_utf8.get_or_intern("readonly");
        allocated_utf8.get_or_intern("return");
        allocated_utf8.get_or_intern("set");
        allocated_utf8.get_or_intern("short");
        allocated_utf8.get_or_intern("static");
        allocated_utf8.get_or_intern("super");
        allocated_utf8.get_or_intern("switch");
        allocated_utf8.get_or_intern("synchronized");
        allocated_utf8.get_or_intern("this");
        allocated_utf8.get_or_intern("throw");
        allocated_utf8.get_or_intern("throws");
        allocated_utf8.get_or_intern("transient");
        allocated_utf8.get_or_intern("true");
        allocated_utf8.get_or_intern("try");
        allocated_utf8.get_or_intern("type");
        allocated_utf8.get_or_intern("typeof");
        allocated_utf8.get_or_intern("using");
        allocated_utf8.get_or_intern("var");
        allocated_utf8.get_or_intern("void");
        allocated_utf8.get_or_intern("volatile");
        allocated_utf8.get_or_intern("while");
        allocated_utf8.get_or_intern("with");
        allocated_utf8.get_or_intern("yield");

        Self {
            allocated_utf8,
            allocated_wtf8: Wtf8Buf::new(),
        }
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        self.allocated_utf8.get_or_intern(s)
    }

    #[inline]
    pub fn add_optional_utf8(&mut self, s: Option<&str>) -> OptionalUtf8Ref {
        match s {
            Some(s) => self.add_utf8(s).into(),
            None => OptionalUtf8Ref::none(),
        }
    }

    #[inline]
    pub fn add_wtf8(&mut self, s: &Wtf8) -> Wtf8Ref {
        let lo = self.allocated_wtf8.len() as u32;
        self.allocated_wtf8.push_wtf8(s);
        let hi = self.allocated_wtf8.len() as u32;
        Wtf8Ref::new_ref(lo, hi)
    }

    #[inline]
    pub fn add_optional_wtf8(&mut self, s: Option<&Wtf8>) -> OptionalWtf8Ref {
        match s {
            Some(s) => self.add_wtf8(s).into(),
            None => OptionalWtf8Ref::new_none(),
        }
    }

    #[inline]
    pub fn get_utf8(&self, id: Utf8Ref) -> &str {
        self.allocated_utf8.resolve(id).unwrap()
    }

    #[inline]
    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<&str> {
        let id = id.to_option()?;
        Some(self.get_utf8(id))
    }

    #[inline]
    pub fn get_wtf8(&self, id: Wtf8Ref) -> &Wtf8 {
        &self
            .allocated_wtf8
            .slice(id.lo() as usize, id.hi() as usize)
    }

    #[inline]
    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<&Wtf8> {
        let id = id.to_option()?;
        Some(self.get_wtf8(id))
    }

    #[inline]
    pub fn utf8_count(&self) -> usize {
        self.allocated_utf8.len()
    }
}
