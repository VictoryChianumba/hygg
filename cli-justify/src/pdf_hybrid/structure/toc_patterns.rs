pub(crate) fn is_title_case_label_word(word: &str) -> bool {
  let token = word.trim_matches(['(', ')', '[', ']', '{', '}', ',', ':', ';']);
  if token.is_empty() {
    return false;
  }

  let mut chars = token.chars();
  let Some(first) = chars.next() else {
    return false;
  };
  first.is_uppercase()
    && chars.all(|ch| ch.is_lowercase() || matches!(ch, '\'' | '-' | '’'))
}

pub(crate) fn is_counter_token(token: &str) -> bool {
  let token = token.trim_matches(['(', ')', ':']);
  if token.is_empty() {
    return false;
  }

  token.chars().all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '-'))
    || token.len() == 1
      && token.chars().next().is_some_and(|ch| ch.is_ascii_uppercase())
    || token
      .chars()
      .all(|ch| matches!(ch, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
}

pub(crate) fn looks_like_toc_entry_prefix(prefix: &str) -> bool {
  let prefix = prefix.trim();
  if prefix.is_empty() {
    return false;
  }

  if prefix.chars().any(|ch| ch.is_ascii_digit())
    && prefix.chars().all(|ch| {
      ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '-' | ' ')
    })
  {
    return true;
  }

  let words: Vec<&str> = prefix.split_whitespace().collect();
  words.len() >= 2
    && is_title_case_label_word(words[0])
    && is_counter_token(words[1])
}

pub(crate) fn looks_like_named_toc_heading(title: &str) -> bool {
  let mut words = title.split_whitespace();
  let Some(first) = words.next() else {
    return false;
  };
  let Some(second) = words.next() else {
    return false;
  };

  is_title_case_label_word(first) && is_counter_token(second)
}

pub(crate) fn merge_counter_into_prefix_if_needed(
  entry_prefix: &str,
  title: &str,
) -> Option<(String, String)> {
  if looks_like_toc_entry_prefix(entry_prefix) {
    return Some((entry_prefix.to_string(), title.to_string()));
  }

  let label = entry_prefix.trim();
  if !is_title_case_label_word(label) {
    return None;
  }

  let mut words = title.split_whitespace();
  let counter = words.next()?;
  if !is_counter_token(counter) {
    return None;
  }
  let rest = words.collect::<Vec<_>>().join(" ");
  if rest.is_empty() {
    return None;
  }

  Some((format!("{entry_prefix}{counter} "), rest))
}

pub(crate) fn looks_like_caption_prefix(prefix: &str) -> bool {
  let mut words = prefix.split_whitespace();
  let Some(label) = words.next() else {
    return false;
  };
  let Some(number) = words.next() else {
    return false;
  };

  label.chars().all(|ch| ch.is_ascii_uppercase()) && is_counter_token(number)
}

pub(crate) fn looks_like_toc_section_marker(token: &str) -> bool {
  let Some((left, right)) = token.split_once('.') else {
    return false;
  };
  if left.is_empty() || right.is_empty() {
    return false;
  }

  token.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '.')
}
