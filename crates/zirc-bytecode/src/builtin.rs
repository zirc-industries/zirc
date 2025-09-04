//! Builtin function identifiers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Show,
    ShowF,
    Prompt,
    Rf,
    Wf,
    Len,
    Push,
    Pop,
    Slice,
}

