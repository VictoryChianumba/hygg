use crate::text_utils::is_ascii_numeric;

const CODE_MARKERS: [&str; 14] = [
  "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", ":=", "+=", "-=", "/*",
  "*/",
];

const PROMPT_PREFIXES: [&str; 7] = ["$", "#", ">", "%", ">>", ">>>", "PS>"];

fn looks_like_prompt_prefix(token: &str) -> bool {
  if PROMPT_PREFIXES.contains(&token) {
    return true;
  }

  token.len() <= 6
    && token.ends_with('>')
    && token
      .chars()
      .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '>'))
}

pub(crate) fn looks_like_command_prompt_line(line: &str) -> bool {
  let trimmed = line.trim();
  if trimmed.is_empty() {
    return false;
  }

  let Some(first_token) = trimmed.split_whitespace().next() else {
    return false;
  };
  if !looks_like_prompt_prefix(first_token) {
    return false;
  }

  trimmed
    .get(first_token.len()..)
    .is_some_and(|rest| !rest.trim_start().is_empty())
}

fn looks_like_fragmented_token_line(trimmed: &str) -> bool {
  if let Some(first) = trimmed.chars().next() {
    if first == '!' && !trimmed.contains(char::is_whitespace) {
      return true;
    }

    if (first == '-' || first == '+')
      && trimmed.chars().nth(1).is_some_and(|ch| ch != ' ' && ch != first)
    {
      return true;
    }

    if first == '.'
      && trimmed.chars().nth(1).is_some_and(|ch| ch.is_ascii_alphanumeric())
    {
      return true;
    }
  }

  false
}

fn looks_like_flag_cluster_line(trimmed: &str) -> bool {
  if !trimmed.starts_with("--") {
    return false;
  }

  let first_token = trimmed.split_whitespace().next().unwrap_or_default();
  if first_token.ends_with(['.', ',', ';', ':']) {
    return false;
  }

  trimmed.split_whitespace().count() <= 3
}

fn looks_like_numbered_label_caption(trimmed: &str) -> bool {
  let mut words = trimmed.split_whitespace();
  let Some(label) = words.next() else {
    return false;
  };
  let Some(number) = words.next() else {
    return false;
  };

  if !label.chars().all(|ch| ch.is_ascii_uppercase()) || label.len() < 5 {
    return false;
  }
  if !number
    .trim_matches([':', '.', ')'])
    .chars()
    .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '-' | '/'))
  {
    return false;
  }

  words.next().is_some()
}

fn symbol_density_looks_like_code(trimmed: &str) -> bool {
  let word_count = trimmed.split_whitespace().count();
  if word_count > 6 {
    return false;
  }

  let mut alpha = 0usize;
  let mut non_space = 0usize;
  let mut punctuation = 0usize;
  for ch in trimmed.chars() {
    if ch.is_whitespace() {
      continue;
    }
    non_space += 1;
    if ch.is_alphabetic() {
      alpha += 1;
    } else if !ch.is_ascii_digit() {
      punctuation += 1;
    }
  }
  if non_space == 0 {
    return false;
  }

  let alpha_ratio = alpha as f64 / non_space as f64;
  let punct_ratio = punctuation as f64 / non_space as f64;
  punct_ratio >= 0.25 && alpha_ratio <= 0.80
}

pub(crate) fn looks_like_code_block_line(line: &str) -> bool {
  let trimmed = line.trim();
  if trimmed.is_empty() {
    return false;
  }

  if looks_like_command_prompt_line(trimmed) {
    return true;
  }
  if looks_like_numbered_label_caption(trimmed) {
    return false;
  }

  if trimmed.starts_with("---")
    || trimmed.starts_with("+++")
    || trimmed.starts_with("@@")
    || trimmed.starts_with("```")
    || trimmed.starts_with("~~~")
  {
    return true;
  }

  if looks_like_flag_cluster_line(trimmed) {
    return true;
  }

  if looks_like_fragmented_token_line(trimmed) {
    return true;
  }

  let word_count = trimmed.split_whitespace().count();
  if word_count <= 2
    && (trimmed.contains('/')
      || trimmed.contains('\\')
      || trimmed.contains('*'))
  {
    return true;
  }

  if CODE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
    return true;
  }

  symbol_density_looks_like_code(trimmed)
}

pub(crate) fn looks_like_toc_entry(trimmed: &str) -> bool {
  let dot_count = trimmed.chars().filter(|&ch| ch == '.').count();
  if dot_count < 4 {
    return false;
  }
  if !(trimmed.contains("...") || trimmed.contains(". .")) {
    return false;
  }

  trimmed.split_whitespace().last().is_some_and(is_ascii_numeric)
}
