# Read grant paths and input names from an app's TOML manifest.
# Strings keep their contents separate from keys, including fenced examples.
# Unsupported names and strings are omitted. This is not a manifest validator.
#
# Usage: awk -f tools/manifest.awk manifest.toml

# Read a quoted string, decoding printable ASCII escapes used in paths and keys.
function read_string(quote,    triple, count, char, escape, digits, number, i) {
  triple = substr(source, position, 3) == quote quote quote
  kind = triple ? "multiline" : "string"
  position += triple ? 3 : 1
  token = ""
  readable = 1
  if (triple && substr(source, position, 1) == "\n") position++

  # Consume the whole string even when its value cannot be represented
  while (position <= length(source)) {
    char = substr(source, position++, 1)
    if (char == quote) {
      count = 1
      if (triple) {
        while (substr(source, position, 1) == quote && count < 5) {
          count++
          position++
        }
      }
      if (!triple || count >= 3) {
        for (i = 3; i < count; i++) token = token quote
        if (!readable) token = "\001"
        return
      }
      for (i = 0; i < count; i++) token = token quote
    } else if (char == "\\" && quote == "\"") {
      escape = substr(source, position++, 1)
      if (triple && escape ~ /[ \t\n]/) {
        while (substr(source, position, 1) ~ /[ \t\n]/) position++
      } else if (escape == "\"" || escape == "\\") {
        token = token escape
      } else if (escape == "x" || escape == "u" || escape == "U") {
        digits = escape == "x" ? 2 : (escape == "u" ? 4 : 8)
        number = 0
        for (i = 0; i < digits; i++) {
          char = tolower(substr(source, position++, 1))
          if (char !~ /^[0-9a-f]$/) readable = 0
          number = number * 16 + index("0123456789abcdef", char) - 1
        }
        if (number >= 32 && number <= 126) {
          token = token sprintf("%c", number)
        } else {
          readable = 0
        }
      } else {
        readable = 0
      }
    } else {
      if (!triple && char == "\n") readable = 0
      token = token char
    }
  }
  kind = "end"
  token = ""
}

# Advance to one token, keeping newlines and dropping whitespace and comments.
function scan(    char) {
  token = ""
  kind = "end"
  while (position <= length(source)) {
    char = substr(source, position, 1)
    if (char == "#") {
      while (position <= length(source) &&
             substr(source, position, 1) != "\n") position++
    } else if (char ~ /[ \t]/) {
      position++
    } else {
      break
    }
  }
  if (position > length(source)) return

  # Quoted values consume their own punctuation and embedded newlines
  if (char == "\"" || char == "'") {
    read_string(char)
    return
  }
  if (index("[]=,.{}\n", char)) {
    kind = token = char
    position++
    return
  }
  kind = "bare"
  while (position <= length(source)) {
    char = substr(source, position, 1)
    if (char ~ /[ \t\n]/ || index("[]=,.{}#\"'", char)) break
    token = token char
    position++
  }
}

# Read a dotted key, keeping quoted dots separate from table separators.
function read_key(    path, part) {
  path = ""
  while (kind == "bare" || kind == "string") {
    part = token ~ /^[A-Za-z0-9_-]+$/ ? token : "?"
    path = path == "" ? part : path SUBSEP part
    scan()
    if (kind != ".") return path
    scan()
  }
  ok = 0
  return "?"
}

# Skip newlines within arrays and TOML 1.1 inline tables.
function skip_newlines() {
  while (kind == "\n") scan()
}

# Return an input fact for a table path with a supported input name.
function input_fact(path,    parts, count) {
  count = split(path, parts, SUBSEP)
  if (count >= 2 && parts[1] == "inputs" &&
      parts[2] ~ /^[a-z][a-z0-9_-]*$/ && length(parts[2]) <= 32) {
    return "input " parts[2] "\n"
  }
  return ""
}

# Read a value and return facts from its arrays, dotted keys and inline tables.
function read_value(path,    facts, grant, key, prefix, parts, count) {
  facts = ""
  count = split(path, parts, SUBSEP)
  prefix = count > 2 || kind == "{" ? input_fact(path) : ""

  # Grant arrays accept readable strings and skip values of other types
  if (kind == "[") {
    if (path == "reads" SUBSEP "paths") grant = "paths"
    if (path == "reads" SUBSEP "optional") grant = "optional"
    scan()
    skip_newlines()
    while (kind != "]") {
      if (grant != "" && (kind == "string" || kind == "multiline") &&
          token != "" && token !~ /[[:space:][:cntrl:]]/) {
        facts = facts grant " " token "\n"
      }
      read_value("?")
      skip_newlines()
      if (!ok) return ""
      if (kind == "]") break
      if (kind != ",") { ok = 0; return "" }
      scan()
      skip_newlines()
    }
    scan()
  } else if (kind == "{") {
    scan()
    skip_newlines()
    while (kind != "}") {
      key = read_key()
      if (!ok || kind != "=") { ok = 0; return "" }
      scan()
      facts = facts read_value(path SUBSEP key)
      skip_newlines()
      if (!ok) return ""
      if (kind == "}") break
      if (kind != ",") { ok = 0; return "" }
      scan()
      skip_newlines()
    }
    scan()
  } else if (kind == "string" || kind == "multiline") {
    scan()
  } else if (kind == "bare") {
    # Scalars such as numbers and dates contribute no nested facts
    while (kind == "bare" || kind == ".") scan()
  } else {
    ok = 0
    return ""
  }
  return prefix facts
}

# Print facts in source order, keeping each input name once.
function emit(facts,    lines, count, i) {
  count = split(facts, lines, "\n")
  for (i = 1; i <= count; i++) {
    if (lines[i] == "") continue
    if (lines[i] ~ /^input / && seen[lines[i]]++) continue
    print lines[i]
  }
}

# Normalize CRLF before strings and comments consume their own lines
{
  sub(/\r$/, "")
  source = source $0 "\n"
}

END {
  # Keep the current table until a complete header replaces it
  position = 1
  ok = 1
  table = ""
  scan()
  while (kind != "end" && ok) {
    skip_newlines()
    if (kind == "end") break
    if (kind == "[") {
      scan()
      array_table = kind == "["
      if (array_table) scan()
      table = read_key()
      if (!ok || kind != "]") break
      scan()
      if (array_table) {
        if (kind != "]") break
        scan()
        table = "?"
      }
      if (kind != "\n" && kind != "end") break
      emit(input_fact(table))
    } else {
      key = read_key()
      if (!ok || kind != "=") break
      scan()
      facts = read_value(table == "" ? key : table SUBSEP key)
      if (!ok || (kind != "\n" && kind != "end")) break
      emit(facts)
    }
  }
}
