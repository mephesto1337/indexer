pub struct Lexer<'a> {
    content: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    fn chomp_while<F>(&mut self, mut f: F) -> &'a str
    where
        F: FnMut(&char) -> bool,
    {
        let index = self
            .content
            .char_indices()
            .take_while(|(_, c)| f(c))
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .expect("At least the first char is valid");
        let token = &self.content[..index];
        self.content = &self.content[index..];
        token
    }

    fn skip_whitespaces(&mut self) {
        if let Some(index) = self
            .content
            .find(|c: char| !c.is_ascii_whitespace() && c != '\n')
        {
            self.content = &self.content[index..];
        }
    }

    pub fn get_next_token(&mut self) -> Option<&'a str> {
        self.skip_whitespaces();
        let first = self.content.chars().next()?;

        if first.is_ascii_digit() {
            Some(self.chomp_while(|c| c.is_ascii_digit() || c == &'.'))
        } else if first.is_ascii_alphabetic() {
            Some(self.chomp_while(|c| c.is_alphanumeric() || c == &'_'))
        } else {
            let mut first = true;
            Some(self.chomp_while(|_| {
                if first {
                    first = false;
                    true
                } else {
                    false
                }
            }))
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next_token()
    }
}
