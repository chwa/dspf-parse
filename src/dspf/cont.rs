use std::io::{self, BufRead};

#[derive(Debug)]
pub struct ContinuedLines<B> {
    inner_iterator: std::io::Lines<B>,
    current_line: usize,
    peeked: Option<Option<io::Result<String>>>,
    continuation: char,
}

impl<B: BufRead> ContinuedLines<B> {
    pub fn from_buf(buf: B) -> Self {
        Self {
            inner_iterator: buf.lines(),
            current_line: 1,
            peeked: None,
            continuation: '+',
        }
    }
}

// TODO: return 'enumerated' lines (where the line numbers refer to the first line in a continuation)
// (to allow reporting real line numbers to the user)

impl<B: BufRead> Iterator for ContinuedLines<B> {
    type Item = io::Result<(usize, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = String::new();
        let line_no: usize = self.current_line;
        let mut continuing = true;

        while continuing {
            continuing = false;

            let current = self
                .peeked
                .take()
                .unwrap_or_else(|| self.inner_iterator.next());
            self.current_line += 1;
            match current {
                Some(Ok(line)) => {
                    buffer.push_str(
                        &line.strip_prefix(self.continuation).get_or_insert(&line),
                    );

                    self.peeked = Some(self.inner_iterator.next());
                    if let Some(Some(Ok(peeked_line))) = &self.peeked {
                        if let Some(first) = peeked_line.chars().next() {
                            continuing = first == self.continuation
                        }
                    }
                }
                Some(Err(e)) => {
                    return Some(Err(e));
                }
                None => {
                    return None;
                }
            }
        }
        Some(Ok((line_no, buffer)))
    }
}
