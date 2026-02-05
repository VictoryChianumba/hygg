fn split_at_char(s: &str, n: usize) -> (&str, Option<&str>) {
  for (char_index, (i, _)) in s.char_indices().enumerate() {
    if char_index == n {
      let (w1, w2) = s.split_at(i);
      return (w1, Some(w2));
    }
  }

  (s, None)
}

fn char_len(s: &str) -> usize {
  s.chars().count()
}

fn split_at_last_whitespace_before(s: &str, max_chars: usize) -> Option<usize> {
  let mut last_ws_byte_idx: Option<usize> = None;
  for (char_index, (byte_idx, ch)) in s.char_indices().enumerate() {
    if char_index >= max_chars {
      break;
    }
    if ch == ' ' || ch == '\t' {
      last_ws_byte_idx = Some(byte_idx);
    }
  }
  last_ws_byte_idx
}

fn drop_one_leading_whitespace(s: &str) -> &str {
  let mut chars = s.chars();
  match chars.next() {
    Some(' ') | Some('\t') => chars.as_str(),
    _ => s,
  }
}

fn leading_whitespace(s: &str) -> &str {
  let mut end = 0usize;
  for (byte_idx, ch) in s.char_indices() {
    if ch != ' ' && ch != '\t' {
      break;
    }
    end = byte_idx + ch.len_utf8();
  }
  &s[..end]
}

fn wrap_line_preserving_whitespace(
  line: &str,
  line_width: usize,
) -> Vec<String> {
  if line_width == 0 {
    return vec![String::new()];
  }

  if char_len(line) <= line_width {
    return vec![line.to_string()];
  }

  let indent = leading_whitespace(line).to_string();
  let indent_chars = char_len(&indent);
  let mut out = Vec::new();

  let mut remainder = line;
  let mut is_first = true;

  loop {
    let available = if is_first {
      line_width
    } else {
      line_width.saturating_sub(indent_chars)
    };

    if available == 0 {
      // Extremely narrow width: fall back to hard splits.
      let (w1, w2) = split_at_char(remainder, 1);
      out.push(if is_first { w1.to_string() } else { format!("{indent}{w1}") });
      remainder = w2.unwrap_or("");
      if remainder.is_empty() {
        break;
      }
      is_first = false;
      continue;
    }

    if char_len(remainder) <= available {
      out.push(if is_first {
        remainder.to_string()
      } else {
        format!("{indent}{remainder}")
      });
      break;
    }

    let split_byte_idx = split_at_last_whitespace_before(remainder, available)
      .unwrap_or_else(|| {
        let (w1, _) = split_at_char(remainder, available);
        w1.len()
      });

    let (chunk, rest) = remainder.split_at(split_byte_idx);
    out.push(if is_first {
      chunk.trim_end_matches([' ', '\t']).to_string()
    } else {
      format!("{indent}{}", chunk.trim_end_matches([' ', '\t']))
    });

    remainder = drop_one_leading_whitespace(rest);
    is_first = false;

    if remainder.is_empty() {
      break;
    }
  }

  out
}

pub fn justify(text: &str, line_width: usize) -> Vec<String> {
  let paragraphs: Vec<&str> = text.split("\n\n").collect();
  let mut lines: Vec<String> = Vec::new();

  for paragraph in paragraphs {
    let raw_words: Vec<&str> = paragraph.split_whitespace().collect();
    let mut words = vec![];

    for mut word in raw_words {
      while let (w1, Some(w2)) = split_at_char(word, line_width) {
        words.push(w1);
        word = w2;
      }

      words.push(word);
    }

    let mut line: Vec<&str> = Vec::new();
    let mut len = 0;

    for word in words {
      // Calculate the length if we add this word
      let word_len = word.len();
      let space_len = if line.is_empty() { 0 } else { 1 };
      let new_len = len + space_len + word_len;

      // If adding this word would exceed the line width and we have words on
      // the line
      if new_len > line_width && !line.is_empty() {
        lines.push(justify_line(&line, line_width));
        line.clear();
        len = 0;
      }

      line.push(word);
      len = if line.len() == 1 { word_len } else { len + space_len + word_len };
    }

    // Add the last line of the paragraph
    if !line.is_empty() {
      lines.push(line.join(" "));
    }

    // Add a blank line after each paragraph to preserve paragraph breaks
    lines.push(String::new());
  }

  lines
}

pub fn wrap_preserve_whitespace(text: &str, line_width: usize) -> Vec<String> {
  let mut out = Vec::new();
  for line in text.split('\n') {
    if line.is_empty() {
      out.push(String::new());
      continue;
    }
    out.extend(wrap_line_preserving_whitespace(line, line_width));
  }
  out
}

fn justify_line(line: &[&str], line_width: usize) -> String {
  let word_len: usize = line.iter().map(|s| s.len()).sum();

  // If the words are already longer than or equal to line width,
  // or if there's only one word, just join them with single spaces
  if word_len >= line_width || line.len() <= 1 {
    return line.join(" ");
  }

  let spaces = line_width - word_len;

  let line_len_div = if (line.len() > 1) { (line.len() - 1) } else { 1 };

  let each_space = spaces / line_len_div;
  let extra_space = spaces % line_len_div;

  let mut justified = String::new();
  for (i, word) in line.iter().enumerate() {
    justified.push_str(word);
    if i < line.len() - 1 {
      let mut space = " ".repeat(each_space);
      if i < extra_space {
        space.push(' ');
      }
      justified.push_str(&space);
    }
  }

  justified
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_handles_long_words() {
    let input_text = r#"some text and a very loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong word but no cause to panic"#;
    let pretty_short_line_width = 10;
    let result = justify(input_text, pretty_short_line_width);
    assert!(!result.is_empty());
  }

  #[test]
  fn test_handles_line_longer_than_width() {
    let input_text =
      "This is a line that is definitely longer than the requested width";
    let result = justify(input_text, 20);
    assert!(!result.is_empty());
    // Should not panic
  }

  #[test]
  fn test_single_word_longer_than_width() {
    let input_text = "supercalifragilisticexpialidocious";
    let result = justify(input_text, 10);
    assert!(!result.is_empty());
    // Word should be split into multiple lines
    assert!(result.len() > 1);
  }

  #[test]
  fn test_normal_justification() {
    // Test with multiple lines to see justification
    let input_text = "This is a test of the justification system. It should properly justify lines that need to be wrapped.";
    let result = justify(input_text, 20);
    assert!(!result.is_empty());

    // Find a line that was justified (not the last line)
    let mut found_justified = false;
    for (i, line) in result.iter().enumerate() {
      if !line.is_empty() && i < result.len() - 2 {
        // Not the last line or blank line
        if line.len() == 20 {
          found_justified = true;
          break;
        }
      }
    }
    assert!(found_justified, "Should have at least one justified line");
  }

  #[test]
  fn wrap_preserves_indentation_and_spacing() {
    let input = "a    b    c";
    let out = wrap_preserve_whitespace(input, 80);
    assert_eq!(out, vec![input.to_string()]);

    let input =
      "    fn main() {    println!(\"hi\");    println!(\"there\"); }";
    let out = wrap_preserve_whitespace(input, 25);
    assert!(out.len() > 1);
    assert!(out[0].starts_with("    "));
    assert!(out[1].starts_with("    "));
  }
}
