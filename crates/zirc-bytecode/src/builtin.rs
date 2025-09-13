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
    // Mathematical functions
    Abs,
    Min,
    Max,
    Pow,
    Sqrt,
    // String functions
    Upper,
    Lower,
    Trim,
    Split,
    Join,
    // Type conversion
    Int,
    Str,
    // Utility functions
    Type,
}

