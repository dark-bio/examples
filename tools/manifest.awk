# Read grants, inputs, outputs and dependencies from an app's TOML manifest.
# Strings keep their contents separate from keys, including fenced examples.
# The default facts omit unsupported names. The task planner uses the same
# parser's ordered nodes, with strings held as octal escapes, including NUL.
# This reads the manifest's tables, not every TOML type or every Ark rule.
#
# Usage: LC_ALL=C awk -f tools/manifest.awk manifest.toml

# Encode bytes for POSIX printf %b without holding a native NUL in awk.
function octal(value,    result, i) {
  for (i = 1; i <= length(value); i++)
    result = result sprintf("\\0%03o", ord[substr(value, i, 1)])
  return result
}

# Decode a non-NUL string used for names and diagnostics.
function plain(value,    result, i, code) {
  for (i = 1; i <= length(value); i += 5) {
    code = substr(value, i + 2, 1) * 64 + substr(value, i + 3, 1) * 8 + \
           substr(value, i + 4, 1)
    result = result (code ? sprintf("%c", code) : "\001")
  }
  return result
}

# Allocate a scalar, array or table in the ordered manifest tree.
function node(type, value) {
  node_type[++node_count] = type
  node_value[node_count] = value
  return node_count
}

# Return one named member without changing the table's order.
function member(parent, key) {
  return members[parent, octal(key)]
}

# Attach one named node to a table in declaration order.
function attach(parent, key, child) {
  members[parent, key] = child
  entries[parent, ++sizes[parent]] = child
  node_name[child] = key
}

# Stop parsing with the current app and declaration in the diagnostic.
function manifest_error(message) {
  print "error: " task_path ": cannot read manifest: " message > "/dev/stderr"
  exit 1
}

# Encode a Unicode scalar as UTF-8 while running awk in the C locale.
function utf8(code) {
  if (code < 128) return sprintf("%c", code)
  if (code < 2048)
    return sprintf("%c%c", 192 + int(code / 64), 128 + code % 64)
  if (code < 65536)
    return sprintf("%c%c%c", 224 + int(code / 4096),
                   128 + int(code / 64) % 64, 128 + code % 64)
  return sprintf("%c%c%c%c", 240 + int(code / 262144),
                 128 + int(code / 4096) % 64,
                 128 + int(code / 64) % 64, 128 + code % 64)
}

# Keep declarations in source order.
function record(path, value,    parts, count, i, parent, child) {
  if (path == "?") return
  count = split(path, parts, SUBSEP)
  if (parts[1] == octal("listing")) return
  parent = manifest_root
  for (i = 1; i <= count; i++) {
    child = members[parent, parts[i]]
    if (!child) {
      child = i == count && value ? value : node("table", "")
      attach(parent, parts[i], child)
      if (!task && i == 2 && node_type[child] == "table")
        declaration_fact(plain(parts[1]), plain(parts[2]))
    }
    parent = child
  }
  return child
}

# Append decoded characters and their lossless octal representation.
function append_string(value) {
  token = token value
  string_bytes = string_bytes octal(value)
}

# Read a quoted string, decoding the escapes used in literal input bindings.
function read_string(quote,    triple, count, char, escape, digits, number, i) {
  triple = substr(source, position, 3) == quote quote quote
  kind = triple ? "multiline" : "string"
  position += triple ? 3 : 1
  token = ""
  string_bytes = ""
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
        for (i = 3; i < count; i++) append_string(quote)
        if (!readable) token = "\001"
        return
      }
      for (i = 0; i < count; i++) append_string(quote)
    } else if (char == "\\" && quote == "\"") {
      escape = substr(source, position++, 1)
      if (triple && escape ~ /[ \t\n]/) {
        while (substr(source, position, 1) ~ /[ \t\n]/) position++
      } else if (escape == "\"" || escape == "\\") {
        append_string(escape)
      } else if (escape == "x" || escape == "u" || escape == "U") {
        digits = escape == "x" ? 2 : (escape == "u" ? 4 : 8)
        number = 0
        for (i = 0; i < digits; i++) {
          char = tolower(substr(source, position++, 1))
          if (char !~ /^[0-9a-f]$/) readable = 0
          number = number * 16 + index("0123456789abcdef", char) - 1
        }
        if (number <= 1114111 && !(number >= 55296 && number <= 57343)) {
          if (number == 0) {
            # Keep NUL escaped because some awk implementations drop native NUL
            string_bytes = string_bytes "\\0000"
            token = token "\001"
          } else {
            append_string(utf8(number))
          }
        } else {
          readable = 0
        }
      } else if (index("btnfre", escape)) {
        number = index("btnfre", escape)
        append_string(substr("\b\t\n\f\r\033", number, 1))
      } else {
        readable = 0
      }
    } else {
      if (!triple && char == "\n") readable = 0
      append_string(char)
    }
  }
  ok = 0
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
    if (kind == "string" && !readable) { ok = 0; return "?" }
    part = kind == "string" ? string_bytes : octal(token)
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

# Print a declaration fact when a table first introduces a supported name.
function declaration_fact(table, name) {
  if (table ~ /^(inputs|outputs|apps)$/ &&
      name ~ /^[a-z][a-z0-9_-]*$/ && length(name) <= 32)
    print substr(table, 1, length(table) - 1), name
}

# Read a value under its full path, keeping quoted dots in their keys.
function read_entry(base, key) {
  return read_value(base == "?" ? "?" : (base == "" ? key : base SUBSEP key))
}

# Read a scalar, array or inline table into the ordered manifest tree.
function read_value(path,    grant, key, values, result) {
  # Grant arrays accept readable strings and skip values of other types
  if (kind == "[") {
    result = node("array", "")
    if (path == octal("reads") SUBSEP octal("paths")) grant = "paths"
    if (path == octal("reads") SUBSEP octal("optional")) grant = "optional"
    scan()
    skip_newlines()
    while (kind != "]") {
      if (!task && grant != "" && (kind == "string" || kind == "multiline") &&
          token != "" && token !~ /[[:space:][:cntrl:]]/) print grant, token
      entries[result, ++sizes[result]] = read_value("?")
      skip_newlines()
      if (!ok) return ""
      if (kind == "]") break
      if (kind != ",") { ok = 0; return "" }
      scan()
      skip_newlines()
    }
    scan()
  } else if (kind == "{") {
    result = record(path, 0)
    if (!result) result = node("table", "")
    scan()
    skip_newlines()
    while (kind != "}") {
      key = read_key()
      if (!ok || kind != "=") { ok = 0; return "" }
      scan()
      read_entry(path, key)
      skip_newlines()
      if (!ok) return ""
      if (kind == "}") break
      if (kind != ",") { ok = 0; return "" }
      scan()
      skip_newlines()
    }
    scan()
  } else if (kind == "string" || kind == "multiline") {
    if (!readable) { ok = 0; return "" }
    result = node("string", string_bytes)
    scan()
  } else if (kind == "bare") {
    # Consume a fractional scalar without treating its integer prefix as a value
    values = token
    scan()
    while (kind == ".") {
      values = values "."
      scan()
      if (kind != "bare") { ok = 0; return "" }
      values = values token
      scan()
    }
    result = node("bare", values)
  } else {
    ok = 0
    return ""
  }
  record(path, result)
  return result
}

# Byte values let octal escapes preserve control characters in bindings.
BEGIN {
  for (byte = 1; byte < 256; byte++) ord[sprintf("%c", byte)] = byte
}

# Normalize CRLF before strings and comments consume their own lines
{
  sub(/\r$/, "")
  source = source $0 "\n"
}

# Parse the buffered document for facts or for the task's ordered nodes.
function parse_manifest(    table, array_table, key) {
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
      if (!ok || kind != "]") { ok = 0; break }
      scan()
      if (array_table) {
        if (kind != "]") { ok = 0; break }
        scan()
      }
      if (kind != "\n" && kind != "end") { ok = 0; break }
      record(table, 0)
    } else {
      key = read_key()
      if (!ok || kind != "=") { ok = 0; break }
      scan()
      read_entry(table, key)
      if (!ok || (kind != "\n" && kind != "end")) { ok = 0; break }
    }
  }
  if (task && (!ok || kind != "end")) manifest_error("syntax")
}

END {
  if (!task) {
    manifest_root = node("table", "")
    parse_manifest()
  }
}
