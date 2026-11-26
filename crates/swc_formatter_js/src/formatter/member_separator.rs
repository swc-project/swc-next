use crate::{Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum MemberBody {
    Class,
    Interface,
    TypeLiteral,
}

impl MemberBody {
    pub(super) fn bare_name_requires_separator(
        self,
        name: Option<&str>,
        following: bool,
        export_can_be_modifier: bool,
    ) -> bool {
        match name {
            // Preserve the pinned class and interface style even at the end.
            Some("get" | "set" | "static") => following || !matches!(self, Self::TypeLiteral),
            // TypeScript can read a bare `export` as a modifier across a newline.
            Some("export") => following && export_can_be_modifier,
            _ => false,
        }
    }
}

/// TypeScript can consume these modifiers across a member's line break.
pub(super) fn ts_member_modifier_needs_separator(
    name: Option<&str>,
    next_name: Option<&str>,
    decorated: bool,
) -> bool {
    matches!(
        (name, next_name),
        (Some("const"), Some("enum")) | (Some("default"), Some("class" | "function" | "interface"))
    ) || name == Some("default") && decorated
}

pub(super) fn ts_member_modifier(name: &str) -> bool {
    matches!(
        name,
        "abstract"
            | "accessor"
            | "async"
            | "const"
            | "declare"
            | "default"
            | "export"
            | "in"
            | "out"
            | "override"
            | "private"
            | "protected"
            | "public"
            | "readonly"
            | "static"
    )
}

#[derive(Clone, Copy)]
pub(super) enum MemberSeparator {
    Omit,
    Semicolon,
    IfBreak,
    IfFlat,
}

impl MemberSeparator {
    pub(super) fn needs_requirement(semicolons: Semicolons) -> bool {
        matches!(semicolons, Semicolons::AsNeeded)
    }

    pub(super) fn select(
        body: MemberBody,
        semicolons: Semicolons,
        following: bool,
        required: bool,
    ) -> Self {
        if required {
            return Self::Semicolon;
        }
        match (body, semicolons, following) {
            (MemberBody::Class, Semicolons::Always, _) | (_, Semicolons::Always, true) => {
                Self::Semicolon
            }
            (_, Semicolons::Always, false) => Self::IfBreak,
            (MemberBody::Class, Semicolons::AsNeeded, _) | (_, Semicolons::AsNeeded, false) => {
                Self::Omit
            }
            (_, Semicolons::AsNeeded, true) => Self::IfFlat,
        }
    }

    pub(super) fn require_if(self, required: bool) -> Self {
        if required { Self::Semicolon } else { self }
    }

    pub(super) fn minimum_width(self, body_always_breaks: bool) -> usize {
        match self {
            Self::Semicolon => ";".len(),
            Self::IfBreak if body_always_breaks => ";".len(),
            _ => 0,
        }
    }

    pub(super) fn write(self, writer: &mut Writer<'_>) {
        match self {
            Self::Omit => {}
            Self::Semicolon => writer.text(";"),
            Self::IfBreak => writer.if_break(";"),
            Self::IfFlat => writer.if_flat(";"),
        }
    }
}
