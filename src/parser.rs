use crate::{
    Aggregate, AggregateKind, Atom, Error, Goal, Literal, Program, Query, Rule, Term, Value,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Name(String),
    Variable(String),
    Text(String),
    Integer(i64),
    Left,
    Brace,
    CloseBrace,
    Colon,
    Equal,
    At,
    Witness,
    Right,
    Comma,
    Dot,
    Rule,
    Query,
    End,
}

struct Parser {
    tokens: Vec<(Token, usize)>,
    position: usize,
    anonymous: usize,
}

fn tokenize(source: &str) -> Result<Vec<(Token, usize)>, Error> {
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut tokens = Vec::new();
    while let Some(character) = chars.next() {
        if character.is_whitespace() {
            if character == '\n' {
                line += 1;
            }
            continue;
        }
        if character == '%' || character == '#' {
            for c in chars.by_ref() {
                if c == '\n' {
                    line += 1;
                    break;
                }
            }
            continue;
        }
        let start_line = line;
        let fail = |message: &str| Error::new(format!("Line {start_line}: {message}"));
        let token = match character {
            '(' => Token::Left,
            '{' => Token::Brace,
            '}' => Token::CloseBrace,
            '=' => Token::Equal,
            '@' => Token::At,
            '~' => Token::Witness,
            ')' => Token::Right,
            ',' => Token::Comma,
            '.' => Token::Dot,
            ':' | '?' if chars.peek() == Some(&'-') => {
                chars.next();
                if character == ':' {
                    Token::Rule
                } else {
                    Token::Query
                }
            }
            ':' => Token::Colon,
            '\'' | '"' => {
                let mut text = String::new();
                loop {
                    let c = chars
                        .next()
                        .ok_or_else(|| fail("Close the quoted value."))?;
                    if c == character {
                        break;
                    }
                    if c == '\\' {
                        let escaped = chars
                            .next()
                            .ok_or_else(|| fail("Complete the escape sequence."))?;
                        text.push(match escaped {
                            'n' => '\n', 'r' => '\r', 't' => '\t', '\\' => '\\', '\'' => '\'', '"' => '"',
                            _ => return Err(fail("Unsupported escape; escape a quote, backslash, newline, carriage return, or tab.")),
                        });
                    } else {
                        if c == '\n' {
                            line += 1;
                        }
                        text.push(c);
                    }
                }
                if character == '\'' {
                    Token::Name(text)
                } else {
                    Token::Text(text)
                }
            }
            c if c.is_ascii_digit() || c == '-' => {
                let mut text = String::from(c);
                while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    text.push(chars.next().unwrap());
                }
                Token::Integer(text.parse().map_err(|_| {
                    fail("Use an integer between -9223372036854775808 and 9223372036854775807.")
                })?)
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut text = String::from(c);
                while chars
                    .peek()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
                {
                    text.push(chars.next().unwrap());
                }
                if c.is_ascii_uppercase() || c == '_' {
                    Token::Variable(text)
                } else {
                    Token::Name(text)
                }
            }
            _ => {
                return Err(fail(&format!(
                    "Unexpected character {character:?}; use facts, Head :- Body, and ?- queries."
                )))
            }
        };
        tokens.push((token, start_line));
    }
    tokens.push((Token::End, line));
    Ok(tokens)
}

impl Parser {
    fn new(source: &str) -> Result<Self, Error> {
        Ok(Self {
            tokens: tokenize(source)?,
            position: 0,
            anonymous: 0,
        })
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.position].0
    }
    fn error(&self, message: impl AsRef<str>) -> Error {
        Error::new(format!(
            "Line {}: {}",
            self.tokens[self.position].1,
            message.as_ref()
        ))
    }
    fn take(&mut self, token: Token) -> bool {
        if self.peek() == &token {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, token: Token, message: &str) -> Result<(), Error> {
        if self.take(token) {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }
    fn term(&mut self) -> Result<Term, Error> {
        if self.take(Token::Witness) {
            let Token::Text(value) = self.peek().clone() else {
                return Err(self.error("A witness needs a quoted identity after ~."));
            };
            self.position += 1;
            return Ok(Term::Constant(Value::Witness(value)));
        }
        let term = match self.peek().clone() {
            Token::Name(value) => Term::Constant(Value::Symbol(value)),
            Token::Text(value) => Term::Constant(Value::Text(value)),
            Token::Integer(value) => Term::Constant(Value::Integer(value)),
            Token::Variable(name) if name == "_" => {
                let name = format!("_${}", self.anonymous);
                self.anonymous += 1;
                Term::Variable(name)
            }
            Token::Variable(name) => Term::Variable(name),
            _ => return Err(self.error(
                "Expected an atom, string, integer, or variable; compound terms are not supported.",
            )),
        };
        self.position += 1;
        Ok(term)
    }
    fn goal(&mut self) -> Result<Goal, Error> {
        let Token::Name(predicate) = self.peek().clone() else {
            return Err(self.error("Expected a predicate name."));
        };
        self.position += 1;
        let mut terms = Vec::new();
        if self.take(Token::Left) {
            loop {
                let term = self.term()?;
                terms.push(term);
                if terms.len() > 64 {
                    return Err(self.error("A relation supports at most 64 arguments."));
                }
                if !self.take(Token::Comma) {
                    break;
                }
            }
            self.expect(
                Token::Right,
                "Close the argument list with ); nested terms are not supported.",
            )?;
        }
        Ok(Goal { predicate, terms })
    }
    fn literal(&mut self) -> Result<Literal, Error> {
        if self.take(Token::At) {
            let goal = self.goal()?;
            if goal.predicate == "fresh" {
                if goal.terms.len() != 1 {
                    return Err(self.error("@fresh needs one output variable."));
                }
                return Ok(Literal::Fresh(goal.terms[0].clone()));
            }
            return Ok(Literal::Builtin(goal));
        }
        if self.peek() == &Token::Name("not".into())
            && !matches!(
                self.tokens[self.position + 1].0,
                Token::Left | Token::Comma | Token::Dot
            )
        {
            self.position += 1;
            return Ok(Literal::Not(self.goal()?));
        }
        if let Token::Name(name) = self.peek().clone() {
            if self.tokens[self.position + 1].0 == Token::Brace {
                let kind = match name.as_str() {
                    "count" => AggregateKind::Count,
                    "sum" => AggregateKind::Sum,
                    "min" => AggregateKind::Min,
                    "max" => AggregateKind::Max,
                    _ => return Err(self.error("Use count, sum, min, or max before an aggregate.")),
                };
                self.position += 2;
                let mut template = vec![self.term()?];
                while self.take(Token::Comma) {
                    if template.len() == 64 {
                        return Err(self.error("Use at most 64 aggregate template terms."));
                    }
                    template.push(self.term()?);
                }
                self.expect(
                    Token::Colon,
                    "Separate the aggregate template and goals with :.",
                )?;
                let body = self.goals()?;
                self.expect(Token::CloseBrace, "Close the aggregate with }.")?;
                self.expect(Token::Equal, "Follow the aggregate with = Result.")?;
                let result = self.term()?;
                return Ok(Literal::Aggregate(Aggregate {
                    kind,
                    template,
                    body,
                    result,
                }));
            }
        }
        Ok(Literal::Relation(self.goal()?))
    }
    fn goals(&mut self) -> Result<Vec<Literal>, Error> {
        let mut goals = vec![self.literal()?];
        while self.take(Token::Comma) {
            if goals.len() == 64 {
                return Err(self.error("Use at most 64 goals in a rule or query."));
            }
            goals.push(self.literal()?);
        }
        Ok(goals)
    }
    fn program(&mut self) -> Result<Program, Error> {
        let mut program = Program::default();
        while self.peek() != &Token::End {
            if self.take(Token::Query) {
                let goals = self.goals()?;
                let mut variables = Vec::new();
                for goal in &goals {
                    for name in literal_variable_order(goal) {
                        if !name.starts_with("_$") && !variables.contains(&name) {
                            variables.push(name);
                        }
                    }
                }
                program.queries.push(Query { goals, variables });
            } else {
                let head = self.goal()?;
                if self.take(Token::Rule) {
                    let body = self.goals()?;
                    program.rules.push(Rule { head, body });
                } else {
                    program.facts.push(head.ground().ok_or_else(|| {
                        self.error("A fact must be ground; put variables in a rule or query.")
                    })?);
                }
            }
            self.expect(Token::Dot, "End each fact, rule, or query with a period.")?;
        }
        program.validate()?;
        Ok(program)
    }
}

fn literal_variable_order(literal: &Literal) -> Vec<String> {
    let terms = match literal {
        Literal::Relation(g) | Literal::Builtin(g) | Literal::Not(g) => g.terms.as_slice(),
        Literal::Aggregate(a) => std::slice::from_ref(&a.result),
        Literal::Fresh(t) => std::slice::from_ref(t),
    };
    terms
        .iter()
        .filter_map(|t| {
            if let Term::Variable(n) = t {
                Some(n.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn parse(source: &str) -> Result<Program, Error> {
    Parser::new(source)?.program()
}

pub fn parse_atom(source: &str) -> Result<Atom, Error> {
    let mut parser = Parser::new(source)?;
    let goal = parser.goal()?;
    parser.expect(Token::Dot, "End the recorded fact with a period.")?;
    if parser.peek() != &Token::End {
        return Err(parser.error("Expected exactly one recorded fact."));
    }
    goal.ground()
        .ok_or_else(|| parser.error("Recorded facts must be ground."))
}
