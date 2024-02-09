use std::cmp::min;
use std::fs;
use std::path::Path;

pub struct QueryList {
    pub queries: Vec<String>,
}

impl QueryList {
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path) //
            .expect("Failed to read input file");

        Self {
            queries: split_queries(&content),
        }
    }
}

/// Split the statements from a multi-statement sql string into a Vec<String>.
/// `sql` is an &str containing some sql statements separated by semicolons.
pub fn split_queries(sql: &str) -> Vec<String> {
    // Taken from https://github.com/jvasile/sql_split/blob/main/src/lib.rs

    let mut ret: Vec<String> = vec![];
    let mut statement: String = "".to_owned();
    let mut encloser: Option<char> = None;
    let mut last_ch = ' ';
    let mut in_line_comment: bool = false;
    let mut in_block_comment: bool = false;
    let mut in_dot_command: bool = false;
    let mut in_paren: bool = false;
    let mut record_statement: bool = false;
    for ch in sql.chars() {
        if !in_line_comment && !in_block_comment {
            statement.push(ch);
        }

        // if we're in a comment, we need to ignore some noise, so
        // let's treat it as a special case.
        if in_line_comment && ch == '\n' {
            statement.push(ch);
            in_line_comment = false;
        }

        match encloser {
            Some(e) => {
                if ch == ']' || e == ch {
                    encloser = None;
                }
            }
            None => match ch {
                '.' => {
                    if statement.len() == 1 && !in_block_comment {
                        in_dot_command = true;
                    }
                }
                '*' => {
                    if !in_line_comment && !in_block_comment {
                        if last_ch == '/' {
                            in_block_comment = true;
                            // unpush the /*
                            statement.pop().unwrap();
                            statement.pop().unwrap();

                            // This one might be controversial.  If
                            // you start a /*comment*/ while in a
                            // .command, sqlite will throw an error.
                            // We are stripping comments, though, so
                            // we can't really reproduce that.  Here,
                            // I just end the .command and delete the
                            // comment.
                            if in_dot_command {
                                record_statement = true;
                            }
                        }
                    }
                }
                '/' => {
                    if in_block_comment && last_ch == '*' {
                        in_block_comment = false;
                    }
                }
                '\n' => {
                    if in_dot_command {
                        record_statement = true;
                        in_dot_command = false;
                    }
                }
                '-' => {
                    if !in_line_comment && !in_block_comment {
                        if last_ch == '-' {
                            in_line_comment = true;

                            // unpush the --
                            statement.pop().unwrap();
                            statement.pop().unwrap();
                        }
                    }
                }
                ';' => {
                    if !in_paren && !in_line_comment && !in_block_comment {
                        record_statement = true;
                    }
                }
                '(' => in_paren = true,
                ')' => in_paren = false,
                '[' | '"' | '\'' | '`' => encloser = Some(ch),
                _ => {}
            },
        }
        last_ch = ch;

        if record_statement {
            statement = statement.trim().to_owned();

            // Push statement if not empty
            if statement != ";" && !statement.is_empty() {
                ret.push(statement.to_owned());
            }
            statement = "".to_owned();
            record_statement = false;
        }
    }

    // Capture anything left over, in case sql doesn't end in
    // semicolon.  Note that if we `break` in the above loop,
    // statement might not be empty, and all the sql left will get
    // tacked on to ret.
    if statement.trim().len() != 0 {
        ret.push(statement.trim().to_string())
    }

    ret
}
