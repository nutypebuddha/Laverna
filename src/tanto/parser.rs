use super::TantoEnv;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
    LParen,
    RParen,
    Comma,
    Func(String),
    Ident(String),
}

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a [u8]) -> Self {
        Lexer { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos] == b' ' {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }
        let c = self.input[self.pos];
        match c {
            b'+' => {
                self.pos += 1;
                Some(Token::Plus)
            }
            b'-' => {
                self.pos += 1;
                Some(Token::Minus)
            }
            b'*' => {
                self.pos += 1;
                if self.pos < self.input.len() && self.input[self.pos] == b'*' {
                    self.pos += 1;
                    Some(Token::Pow)
                } else {
                    Some(Token::Mul)
                }
            }
            b'/' => {
                self.pos += 1;
                Some(Token::Div)
            }
            b'%' => {
                self.pos += 1;
                Some(Token::Mod)
            }
            b'^' => {
                self.pos += 1;
                Some(Token::Pow)
            }
            b'(' => {
                self.pos += 1;
                Some(Token::LParen)
            }
            b')' => {
                self.pos += 1;
                Some(Token::RParen)
            }
            b',' => {
                self.pos += 1;
                Some(Token::Comma)
            }
            b'0'..=b'9' | b'.' => self.read_number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.read_ident(),
            _ => None,
        }
    }

    fn read_number(&mut self) -> Option<Token> {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == b'.')
        {
            self.pos += 1;
        }
        if self.pos < self.input.len()
            && (self.input[self.pos] == b'e' || self.input[self.pos] == b'E')
        {
            self.pos += 1;
            if self.pos < self.input.len()
                && (self.input[self.pos] == b'+' || self.input[self.pos] == b'-')
            {
                self.pos += 1;
            }
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let s = std::str::from_utf8(&self.input[start..self.pos]).ok()?;
        let val = s.parse::<f64>().ok()?;
        Some(Token::Num(val))
    }

    fn read_ident(&mut self) -> Option<Token> {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.input[start..self.pos]).ok()?;
        let funcs: &[&str] = &[
            "sqrt",
            "sin",
            "cos",
            "tan",
            "asin",
            "acos",
            "atan",
            "atan2",
            "abs",
            "exp",
            "ln",
            "log10",
            "log2",
            "hypot",
            "pow",
            "round",
            "floor",
            "ceil",
            "min",
            "max",
            "clamp",
            "sum",
            "avg",
            "rad2deg",
            "deg2rad",
            "norm",
            "f_to_c",
            "c_to_f",
            "c_to_k",
            "f_to_k",
            "k_to_c",
            "k_to_f",
            "mi_to_km",
            "km_to_mi",
            "mph_to_kmh",
            "kmh_to_mph",
            "lb_to_kg",
            "kg_to_lb",
            "ft_to_m",
            "in_to_cm",
            "mph_to_ms",
            "ms_to_mph",
        ];
        if funcs.contains(&s) {
            Some(Token::Func(s.to_string()))
        } else {
            Some(Token::Ident(s.to_string()))
        }
    }
}

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    env: &'a TantoEnv,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, env: &'a TantoEnv) -> Self {
        Parser {
            tokens,
            pos: 0,
            env,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    fn parse_expr(&mut self) -> Option<f64> {
        let mut left = self.parse_term()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left += right;
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut left = self.parse_power()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Mul => {
                    self.advance();
                    let right = self.parse_power()?;
                    left *= right;
                }
                Token::Div => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return None;
                    }
                    left /= right;
                }
                Token::Mod => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return None;
                    }
                    left %= right;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_power(&mut self) -> Option<f64> {
        let base = self.parse_unary()?;
        if let Some(Token::Pow) = self.peek() {
            self.advance();
            let exp = self.parse_power()?; // Right-associative: 2^3^2 = 2^(3^2) = 512
            Some(base.powf(exp))
        } else {
            Some(base)
        }
    }

    fn parse_unary(&mut self) -> Option<f64> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let val = self.parse_atom()?;
                Some(-val)
            }
            Some(Token::Plus) => {
                self.advance();
                self.parse_atom()
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Option<f64> {
        let tok = self.advance()?;
        match tok {
            Token::Num(val) => Some(val),
            Token::LParen => {
                let val = self.parse_expr()?;
                if self.advance() != Some(Token::RParen) {
                    return None;
                }
                Some(val)
            }
            Token::Func(name) => {
                if self.advance() != Some(Token::LParen) {
                    return None;
                }
                let mut args = Vec::new();
                if self.peek() != Some(Token::RParen) {
                    args.push(self.parse_expr()?);
                    while self.peek() == Some(Token::Comma) {
                        self.advance();
                        args.push(self.parse_expr()?);
                    }
                }
                if self.advance() != Some(Token::RParen) {
                    return None;
                }
                eval_func(&name, &args)
            }
            Token::Ident(name) => self.env.get(&name).copied(),
            _ => None,
        }
    }
}

fn eval_func(name: &str, args: &[f64]) -> Option<f64> {
    match name {
        "sqrt" => args.first().map(|x| x.sqrt()),
        "sin" => args.first().map(|x| x.sin()),
        "cos" => args.first().map(|x| x.cos()),
        "tan" => args.first().map(|x| x.tan()),
        "asin" => args.first().map(|x| x.asin()),
        "acos" => args.first().map(|x| x.acos()),
        "atan" => args.first().map(|x| x.atan()),
        "atan2" => {
            if args.len() >= 2 {
                Some(args[0].atan2(args[1]))
            } else {
                None
            }
        }
        "abs" => args.first().map(|x| x.abs()),
        "exp" => args.first().map(|x| x.exp()),
        "ln" => args.first().map(|x| x.ln()),
        "log10" => args.first().map(|x| x.log10()),
        "log2" => args.first().map(|x| x.log2()),
        "hypot" => {
            if args.len() >= 2 {
                Some(args[0].hypot(args[1]))
            } else {
                None
            }
        }
        "pow" => {
            if args.len() >= 2 {
                Some(args[0].powf(args[1]))
            } else {
                None
            }
        }
        "round" => args.first().map(|x| x.round()),
        "floor" => args.first().map(|x| x.floor()),
        "ceil" => args.first().map(|x| x.ceil()),
        "min" => {
            if args.is_empty() {
                None
            } else {
                Some(args.iter().cloned().fold(f64::INFINITY, f64::min))
            }
        }
        "max" => {
            if args.is_empty() {
                None
            } else {
                Some(args.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
            }
        }
        "clamp" => {
            if args.len() >= 3 {
                let val = args[0].max(args[1]).min(args[2]);
                Some(val)
            } else {
                None
            }
        }
        "sum" => Some(args.iter().sum()),
        "avg" => {
            if args.is_empty() {
                None
            } else {
                Some(args.iter().sum::<f64>() / args.len() as f64)
            }
        }
        "rad2deg" => args.first().map(|x| x.to_degrees()),
        "deg2rad" => args.first().map(|x| x.to_radians()),
        "norm" => {
            if args.len() >= 2 {
                Some((args[0] * args[0] + args[1] * args[1]).sqrt())
            } else {
                None
            }
        }
        "f_to_c" => args.first().map(|f| (f - 32.0) * 5.0 / 9.0),
        "c_to_f" => args.first().map(|c| c * 9.0 / 5.0 + 32.0),
        "c_to_k" => args.first().map(|c| c + 273.15),
        "f_to_k" => args.first().map(|f| (f - 32.0) * 5.0 / 9.0 + 273.15),
        "k_to_c" => args.first().map(|k| k - 273.15),
        "k_to_f" => args.first().map(|k| (k - 273.15) * 9.0 / 5.0 + 32.0),
        "mi_to_km" => args.first().map(|mi| mi * 1.609344),
        "km_to_mi" => args.first().map(|km| km / 1.609344),
        "mph_to_kmh" => args.first().map(|mph| mph * 1.609344),
        "kmh_to_mph" => args.first().map(|kmh| kmh / 1.609344),
        "lb_to_kg" => args.first().map(|lb| lb * 0.453592),
        "kg_to_lb" => args.first().map(|kg| kg / 0.453592),
        "ft_to_m" => args.first().map(|ft| ft * 0.3048),
        "in_to_cm" => args.first().map(|inches| inches * 2.54),
        "mph_to_ms" => args.first().map(|mph| mph * 0.44704),
        "ms_to_mph" => args.first().map(|ms| ms / 0.44704),
        _ => None,
    }
}

pub fn eval_math(input: &[u8], env: &TantoEnv) -> Option<f64> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next_token() {
        tokens.push(tok);
    }
    if tokens.is_empty() {
        return None;
    }
    let mut parser = Parser::new(tokens, env);
    parser.parse_expr()
}

pub fn eval_op_format(input: &[u8], env: &TantoEnv) -> Option<f64> {
    let input = trim(input);
    if input.is_empty() {
        return None;
    }

    let (op, rest) = split_token(input)?;
    let op_str = std::str::from_utf8(op).ok()?;
    let rest = trim(rest);

    let mut args = Vec::new();
    if !rest.is_empty() {
        for token in rest.split(|&b| b == b' ') {
            let token = trim(token);
            if token.is_empty() {
                continue;
            }
            if let Some(v) = eval_math(token, env) {
                args.push(v);
            }
        }
    }

    super::math::compute_named(op_str, &args)
}

fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len() && (s[start] == b' ' || s[start] == b'\t') {
        start += 1;
    }
    let mut end = s.len();
    while end > start && (s[end - 1] == b' ' || s[end - 1] == b'\t') {
        end -= 1;
    }
    &s[start..end]
}

fn split_token(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let s = trim(s);
    if s.is_empty() {
        return None;
    }
    let mut i = 0;
    while i < s.len() && s[i] != b' ' && s[i] != b'\t' {
        i += 1;
    }
    if i >= s.len() {
        return Some((s, &[]));
    }
    Some((&s[..i], &s[i..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_math_basic() {
        let env = TantoEnv::new();
        assert_eq!(eval_math(b"2+3", &env), Some(5.0));
        assert_eq!(eval_math(b"10-4", &env), Some(6.0));
        assert_eq!(eval_math(b"3*4", &env), Some(12.0));
        assert_eq!(eval_math(b"12/4", &env), Some(3.0));
    }

    #[test]
    fn test_eval_math_precedence() {
        let env = TantoEnv::new();
        assert_eq!(eval_math(b"2+3*4", &env), Some(14.0));
        assert_eq!(eval_math(b"(2+3)*4", &env), Some(20.0));
    }

    #[test]
    fn test_eval_math_negative() {
        let env = TantoEnv::new();
        assert_eq!(eval_math(b"-5", &env), Some(-5.0));
        assert_eq!(eval_math(b"-2^2", &env), Some(4.0));
    }

    #[test]
    fn test_eval_math_functions() {
        let env = TantoEnv::new();
        assert!((eval_math(b"sqrt(144)", &env).unwrap() - 12.0).abs() < 1e-10);
        assert!((eval_math(b"sin(0)", &env).unwrap() - 0.0).abs() < 1e-10);
        assert!((eval_math(b"cos(0)", &env).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_op_format() {
        let env = TantoEnv::new();
        assert_eq!(eval_op_format(b"add 2 3", &env), Some(5.0));
        assert_eq!(eval_op_format(b"mul 3 4", &env), Some(12.0));
    }

    // ── D3 regression: exponent chain must not drop terms ──

    #[test]
    fn d3_exponent_chain_right_associative() {
        let env = TantoEnv::new();
        // 2^3^2 = 2^(3^2) = 2^9 = 512 (right-associative)
        let v = eval_math(b"2^3^2", &env).unwrap();
        assert!((v - 512.0).abs() < 1e-6, "expected 512, got {v}");

        // ** is the same operator
        let v = eval_math(b"2**3**2", &env).unwrap();
        assert!((v - 512.0).abs() < 1e-6, "expected 512, got {v}");

        // Explicit parens still work
        assert!((eval_math(b"(2^3)^2", &env).unwrap() - 64.0).abs() < 1e-6);
        assert!((eval_math(b"2^(3^2)", &env).unwrap() - 512.0).abs() < 1e-6);
        assert!((eval_math(b"2^3", &env).unwrap() - 8.0).abs() < 1e-6);
    }
}
