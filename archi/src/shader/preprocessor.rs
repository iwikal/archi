use std::collections::{BTreeMap, BTreeSet};

type CowStr = std::borrow::Cow<'static, str>;

type Source = super::ShaderSource;

#[derive(Debug, PartialEq)]
pub enum Error {
    Format,
    NotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Format => write!(f, "format error"),
            Self::NotFound => write!(f, "header not found"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Default)]
pub struct Preprocessor {
    headers: BTreeMap<&'static str, &'static str>,
    expanded: BTreeMap<&'static str, CowStr>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_header(&mut self, source: Source) {
        self.headers.insert(source.name, source.body);
    }

    fn parse_include_name(include: &str) -> Result<&str, Error> {
        let mut words = include.split_whitespace();

        match (words.next(), words.next()) {
            (Some(word), None) => {
                let mut chars = word.chars();
                match (chars.next(), chars.next_back()) {
                    (Some('"'), Some('"')) => Ok(chars.as_str()),
                    _ => Err(Error::Format),
                }
            }
            _ => Err(Error::Format),
        }
    }

    pub fn expand(&mut self, source: &Source) -> Result<CowStr, Error> {
        self.expand_recursive(source, &mut Default::default())
    }

    fn expand_recursive(
        &mut self,
        source: &Source,
        included_names: &mut BTreeSet<&str>,
    ) -> Result<CowStr, Error> {
        if let Some(expanded) = self.expanded.get(source.name) {
            return Ok(expanded.clone());
        }

        let pragma = "#pragma include ";
        let has_pragma = source
            .body
            .lines()
            .any(|l| l.starts_with(pragma));

        let value: CowStr = if has_pragma {
            let mut buf = String::with_capacity(source.body.len());

            for line in source.body.lines() {
                if line.starts_with(pragma) {
                    let include = &line[pragma.len()..];
                    let name = Self::parse_include_name(include)?;

                    if included_names.contains(name) {
                        continue;
                    }

                    let body =
                        *self.headers.get(name).ok_or(Error::NotFound)?;

                    included_names.insert(name);
                    let expanded_header = self.expand_recursive(
                        &Source { name, body },
                        included_names,
                    )?;

                    buf.push_str(&expanded_header);
                    match expanded_header.chars().next_back() {
                        Some('\n') => (),
                        _ => buf.push('\n'),
                    }
                } else {
                    buf.push_str(line);
                    buf.push('\n');
                }
            }

            buf.into()
        } else {
            source.body.into()
        };

        self.expanded.insert(source.name, value.clone());

        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sloc(i: &str) -> impl Iterator<Item = &str> {
        i.lines().filter(|&l| l != "")
    }

    #[test]
    fn noop() {
        let mut p = Preprocessor::new();
        let body = "one\ntwo\nthree";
        let expanded = p.expand(&Source {
            name: "foo.glsl",
            body,
        });

        assert_eq!(expanded.ok(), Some(body.into()));
    }

    #[test]
    fn once() {
        let mut p = Preprocessor::new();
        p.add_header(Source {
            name: "test.glsl",
            body: "#define SUCCESS=1",
        });
        let expanded = p.expand(&Source {
            name: "foo.glsl",
            body: "one\ntwo\n#pragma include \"test.glsl\"\nthree",
        });

        let expected = "one\n\
                        two\n\
                        #define SUCCESS=1\n\
                        three\n";
        assert_eq!(expanded.ok(), Some(expected.into()));
    }

    #[test]
    fn twice() {
        let mut p = Preprocessor::new();
        p.add_header(Source {
            name: "test.glsl",
            body: "#define SUCCESS=1",
        });
        let expanded = p.expand(&Source {
            name: "foo.glsl",
            body: "one\n\
            two\n\
            #pragma include \"test.glsl\"\n\
            #pragma include \"test.glsl\"\n\
            three",
        });

        let expected = "one\n\
                        two\n\
                        #define SUCCESS=1\n\
                        three\n";
        assert_eq!(expanded.ok(), Some(expected.into()));
    }

    #[test]
    fn multiple() {
        let mut p = Preprocessor::new();
        p.add_header(Source {
            name: "alpha.glsl",
            body: "#define ALPHA=1",
        });
        p.add_header(Source {
            name: "beta.glsl",
            body: "#define BETA=1",
        });
        let expanded = p.expand(&Source {
            name: "foo.glsl",
            body: "one\n\
                two\n\
                #pragma include \"alpha.glsl\"\n\
                #pragma include \"beta.glsl\"\n\
                three\n",
        });

        let expected = "one\n\
                        two\n\
                        #define ALPHA=1\n\
                        #define BETA=1\n\
                        three\n";
        assert_eq!(expanded.ok(), Some(expected.into()));
    }

    #[test]
    fn nested() -> anyhow::Result<()> {
        let mut p = Preprocessor::new();
        p.add_header(Source {
            name: "alpha.glsl",
            body: "#define ALPHA=1\n#pragma include \"beta.glsl\"",
        });
        p.add_header(Source {
            name: "beta.glsl",
            body: "#define BETA=1",
        });

        let expanded = &p.expand(&Source {
            name: "foo.glsl",
            body: "one\n\
                two\n\
                #pragma include \"alpha.glsl\"\n\
                three\n",
        })?;

        let expected = "one\n\
                        two\n\
                        #define ALPHA=1\n\
                        #define BETA=1\n\
                        three\n";

        dbg!(expanded);
        dbg!(expected);
        assert!(sloc(expanded).eq(sloc(expected)));

        Ok(())
    }

    #[test]
    fn recursive() -> anyhow::Result<()> {
        let mut p = Preprocessor::new();
        p.add_header(Source {
            name: "self.glsl",
            body: "#pragma include \"self.glsl\"",
        });

        let expanded = &p.expand(&Source {
            name: "foo.glsl",
            body: "foo\n#pragma include \"self.glsl\"",
        })?;

        let expected = "foo";

        dbg!(expanded);
        dbg!(expected);
        assert!(sloc(expanded).eq(sloc(expected)));

        Ok(())
    }

    #[test]
    fn directed_acyclic_graph() -> anyhow::Result<()> {
        let mut p = Preprocessor::new();
        p.add_header(Source {
            name: "node.glsl",
            body: "#define NODE=1\n\
                #pragma include \"leaf.glsl\"",
        });
        p.add_header(Source {
            name: "leaf.glsl",
            body: "#define LEAF=1",
        });

        let expanded = &p.expand(&Source {
            name: "foo.glsl",
            body: "one\n\
            two\n\
            #pragma include \"node.glsl\"\n\
            #pragma include \"leaf.glsl\"\n\
            three",
        })?;

        let expected = "one\n\
                        two\n\
                        #define NODE=1\n\
                        #define LEAF=1\n\
                        three\n";

        dbg!(expanded);
        dbg!(expected);
        assert!(sloc(expanded).eq(sloc(expected)));

        Ok(())
    }

    #[test]
    fn not_found() {
        let mut p = Preprocessor::new();
        match p.expand(&Source {
            name: "foo.glsl",
            body: "one\n\
            two\n\
            #pragma include \"test.glsl\"\n\
            three",
        }) {
            Ok(_) => panic!("expected NotFound error"),
            Err(e) => assert_eq!(e, Error::NotFound),
        };
    }
}
