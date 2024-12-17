use proc_macro as pm;

#[derive(Debug)]
enum ParseError {
    IdentNotFound
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
enum Visibility {
    Pub,
    PubCrate,
    PubSuper,
    Private,
}

impl Visibility {
    fn to_str(&self) -> Option<&str> {
        Some(match self {
            Visibility::Pub => "pub",
            Visibility::PubCrate => "pub(crate)",
            Visibility::PubSuper => "pub(super)",
            Visibility::Private => return None,
        })
    }
}

macro_rules! tokenstream {
    () => {
        pm::TokenStream::new()
    };
}

#[derive(Debug)]
struct ParsedTokenStream {
    visibility: Visibility,
    name: pm::Ident,
    generics: Option<Vec<pm::TokenTree>>,
    data: Option<Vec<Vec<pm::TokenTree>>>,
}

impl ParsedTokenStream {
    fn name(&self) -> pm::Ident {
        self.name.clone()
    }

    fn generics(&self) -> Option<String> {
        if let Some(ref generics) = self.generics {
            Some(generics.iter().map(|tree| tree.to_string()).collect::<String>())
        } else { None }
    }

    fn lifetime(&self) -> Option<Vec<pm::Ident>> {
        if let Some(ref generics) = self.generics {
            generics
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    let cond = match t {
                        proc_macro::TokenTree::Punct(punct) => punct.as_char() == '\'',
                        _ => false
                    };
                    if cond {
                        generics.get(i)
                    } else { None }
                })
                .map(|t| match t {
                    proc_macro::TokenTree::Ident(ident) => Some(ident.clone()),
                    _ => None
                })
                .collect::<Option<Vec<_>>>()
        } else { None }
    }

    fn fields(&self) -> Option<Vec<pm::TokenTree>> {
        if let Some(ref data) = self.data {
            Some(data.iter().map(|tree| tree[0].clone()).collect())
        } else { None }
    }

    fn into_token_stream(&self) -> pm::TokenStream {
        let _visibility = self.visibility.to_str();
        let _name = self.name();
        let _generics = self.generics();
        let _lifetime = self.lifetime();
        let _fnames = self.fields();

        tokenstream!()
    }
}

struct Cursor {
    buffer: Vec<pm::TokenTree>,
    offset: usize
}

impl Cursor {
    fn new(ts: pm::TokenStream) -> Self {
        Self {
            buffer: ts.into_iter().collect(),
            offset: 0,
        }
    }

    fn parse(&mut self) -> Result<ParsedTokenStream, ParseError> {
        let mut visibility = Visibility::Private;
        let mut name: Option<pm::Ident> = None;
        let mut generics: Option<Vec<pm::TokenTree>> = None;
        let mut data: Option<Vec<Vec<pm::TokenTree>>> = None;

        while self.offset < self.buffer.len() {
            match &self.buffer[self.offset] {
                pm::TokenTree::Group(group) => {
                    let group_data = group.stream().into_iter().collect::<Vec<_>>();
                    // what's better? to include ',', or not?
                    let fields = group_data.split(|tree| {
                        match tree {
                            pm::TokenTree::Punct(punct) => punct.as_char() == ',',
                            _ => false
                        }
                    }).map(|trees| trees.to_vec()).collect::<Vec<_>>();
                    data.replace(fields);
                },
                pm::TokenTree::Ident(ident) => {
                    match ident.to_string().as_str() {
                        "pub" => {
                            if let pm::TokenTree::Group(g) = &self.buffer[self.offset + 1] {
                                if g.delimiter() == pm::Delimiter::Parenthesis {
                                    g.stream().into_iter().for_each(|tree| {
                                        if let pm::TokenTree::Ident(v) = tree {
                                            if v.to_string() == "crate" {
                                                visibility = Visibility::PubCrate;
                                            } else if v.to_string() == "super" {
                                                visibility = Visibility::PubSuper;
                                            }
                                        }
                                    });
                                }
                            } else { visibility = Visibility::Pub }
                        }
                        "struct" | "enum" => {
                            self.offset += 1;
                            let pm::TokenTree::Ident(n) = &self.buffer[self.offset] else { continue };
                            name.replace(n.clone());
                        }
                        _ => {}
                    }
                },
                pm::TokenTree::Punct(punct) => {
                    match punct.as_char() {
                        '<' => {
                            let  closing = &self.buffer[self.offset..].iter().position(|tree| {
                                match tree {
                                    pm::TokenTree::Punct(punct) => {
                                        punct.as_char() == '>'
                                    },
                                    _ => false
                                }
                            });
                            if let Some(closing) = closing {
                                generics.replace(
                                    self
                                        // what's better? to include '<' & '>', or not?
                                        .buffer[self.offset + 1..*closing + self.offset]
                                        .iter()
                                        .cloned()
                                        .collect::<Vec<_>>()
                                );
                            } else { continue }
                        },
                        _ => {}
                    }
                },
                pm::TokenTree::Literal(_) => {},
            }
            self.offset += 1;
        }

        let Some(name) = name else { return Err(ParseError::IdentNotFound) };

        Ok(ParsedTokenStream {
            visibility,
            name,
            generics,
            data,
        })
    }
}

#[proc_macro_derive(Micro)]
pub fn derive_micro(ts: pm::TokenStream) -> pm::TokenStream {
    let mut cursor = Cursor::new(ts);
    let parsed_token = cursor.parse().unwrap();

    eprintln!("{:#?}", parsed_token);

    parsed_token.into_token_stream()
}

