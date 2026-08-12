//! Minimal deterministic CSV writer (docs/02 V2 "richer export formats").
//! Hand-rolled to avoid a new dependency; RFC-4180 quoting: fields
//! containing a comma, double quote, or newline are quoted and embedded
//! quotes are doubled. CRLF line endings per the RFC.

/// Serialize `rows` (parallel to `headers`) as CSV.
pub fn write_csv(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(&escape_row(headers));
    for row in rows {
        out.push_str(&escape_row(row));
    }
    out
}

fn escape_row(fields: &[String]) -> String {
    let mut line = String::new();
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            line.push('"');
            line.push_str(&field.replace('"', "\"\""));
            line.push('"');
        } else {
            line.push_str(field);
        }
    }
    line.push_str("\r\n");
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_csv_quotes_and_escapes() {
        let out = write_csv(
            &["name".to_string(), "score".to_string()],
            &[
                vec!["a,b".to_string(), "1".to_string()],
                vec!["say \"hi\"".to_string(), "2".to_string()],
                vec!["line1\nline2".to_string(), "3".to_string()],
            ],
        );
        assert_eq!(
            out,
            "name,score\r\n\"a,b\",1\r\n\"say \"\"hi\"\"\",2\r\n\"line1\nline2\",3\r\n"
        );
    }

    #[test]
    fn write_csv_plain_fields_unquoted() {
        let out = write_csv(&["a".to_string()], &[vec!["plain".to_string()]]);
        assert_eq!(out, "a\r\nplain\r\n");
    }

    #[test]
    fn write_csv_empty_rows() {
        let out = write_csv(&["h".to_string()], &[]);
        assert_eq!(out, "h\r\n");
    }
}
