use crate::errors::*;
use chumsky::prelude::*;
use std::fs;

// Error handling
type Result<T> = std::result::Result<T, AcfError>;

// Parse primitives
/// Representation of an ACF's file content
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Acf {
    /// A list of entries. Valve ACF files should have at least `AppState`
    entries: Vec<Entry>,
}

/// Representation of an individual ACF entry
#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Entry {
    /// Name of the entry
    name: String,

    // A list of expressions
    expressions: Vec<Expr>,

    // A list of sub-entries
    entries: Vec<Entry>,
}

/// Representation of an individual ACF expression (of form "*."\s+"*.")
#[derive(Clone, Debug, PartialEq, Eq)]
struct Expr {
    /// Name of the expression
    name: String,

    /// Value of the expression
    value: String,
}

/// ACF file parser
///
/// An ACF file is just a list of ACF entries. The current implementation returns a vector of
/// entries, but expects a single root entry. It will not parse files that have additional entries given
pub fn parse_acf(path: &str) -> Result<Acf> {
    let contents = match fs::read_to_string(path) {
        Ok(val) => val,
        Err(_) => return Err(AcfError::Read(path.into())),
    };

    let entries = match acf_parser().parse(&contents).into_result() {
        Ok(val) => val,
        Err(e) => {
            e.into_iter()
                .for_each(|err| println!("Parse error: {}", err));
            return Err(AcfError::Parse(ParseError::Unknown));
        }
    };

    Ok(Acf { entries })
}

/// ACF parser
///
/// A wrapper for the entry parser that allows multiple entries to be defined within the file.
/// Will parse until the end of the file is reached
fn acf_parser<'src>() -> impl Parser<'src, &'src str, Vec<Entry>> {
    entry_parser()
        .padded()
        .repeated()
        .collect::<Vec<_>>()
        .then_ignore(end())
        .map(|entries| entries)
}

/// Entry parser
///
/// Entries start with a string literal followed by an opening brace (i.e., '{'). Entries are
/// expected to have a list of expressions, followed by a list of sub-entries. This ordering
/// is currently enforced
fn entry_parser<'src>() -> impl Parser<'src, &'src str, Entry> {
    recursive(|rec_parser| {
        str_parser()
            .padded()
            .then_ignore(just("{").padded())
            .then(expr_parser().padded().repeated().collect::<Vec<_>>())
            .then(rec_parser.padded().repeated().collect::<Vec<_>>())
            .then_ignore(just("}").padded())
            .map(|((name, expressions), entries)| Entry {
                name,
                expressions,
                entries,
            })
            .boxed()
    })
}

/// Expression parser
///
/// Expressions are formed by two string literals delimited by some whitespace. There are no
/// constraints as to what may form entries (will match up until next quote), so you may get
/// strange resulting expressions if the input file is incorrectly formatted
fn expr_parser<'src>() -> impl Parser<'src, &'src str, Expr> {
    str_parser()
        .padded()
        .then(str_parser())
        .padded()
        .map(|(str1, str2)| Expr {
            name: str1,
            value: str2,
        })
}

/// String literal parser
fn str_parser<'src>() -> impl Parser<'src, &'src str, String> {
    just('"')
        .ignore_then(none_of('"').repeated().to_slice())
        .then_ignore(just('"'))
        .padded()
        .map(|val: &str| val.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_run() {
        let result = parse_acf("./acfs/simple.acf");
        assert!(result.is_ok());
    }

    #[test]
    fn simple() {
        let result = parse_acf("./acfs/simple.acf");
        assert!(result.is_ok());
        let result = result.unwrap();
        let root_entry = &result.entries[0];
        assert_eq!(root_entry.name, "AppState");
        let id_expr = &root_entry.expressions[0];
        assert_eq!(id_expr.name, "appid");
        assert_eq!(id_expr.value, "730");
    }

    #[test]
    fn full() {
        let result = parse_acf("./acfs/appmanifest_730.acf");
        assert!(result.is_ok());
        let result = result.unwrap();
        let root_entry = &result.entries[0];
        assert_eq!(root_entry.name, "AppState");
        let id_expr = &root_entry.expressions[0];
        assert_eq!(id_expr.name, "appid");
        assert_eq!(id_expr.value, "730");
        let name_expr = &root_entry.expressions[3];
        assert_eq!(name_expr.name, "name");
        assert_eq!(name_expr.value, "Counter-Strike 2");
    }
}
