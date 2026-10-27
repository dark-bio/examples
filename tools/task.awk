# Plan a local task from manifests collected by run.sh, using manifest.awk.
# The plan checks references and cycles, and leaves value checks to the Ark.
# Each pin runs separately, without the Ark's merged count or shared runs.
#
# Usage: invoked by tools/run.sh with -v task=pins or -v task=plan.

# Stop with the path of the app whose graph cannot be run.
function fail(path, message) {
  print "error: " path ": " message > "/dev/stderr"
  exit 1
}

# Read a named string from the shared manifest tree.
function field(table, key) {
  return plain(node_value[member(table, key)])
}

# Keep filesystem components and plan fields safe to use as local names.
function local_name(item, path,    name) {
  name = plain(node_name[item])
  if (name !~ /^[a-z][a-z0-9_-]*$/) fail(path, "unsupported local name")
  return name
}

# Read each module once, expanding occurrences in depth-first manifest order.
function load_app(digest, path, depth,    app, file, line, root, pins, i, pin, name, child) {
  if (ancestors[digest]) fail(path, "cycle in module dependencies")
  if (!modules[digest]) fail(path, "no module found for sha256 " digest)
  if (++app_count > 1024 || depth >= 128)
    fail(path, "local expansion exceeds 1024 apps or 128 levels")
  app = app_count
  paths[app] = path
  module_ids[app] = modules[digest]
  if (!manifests[digest]) {
    file = stage "/modules/" modules[digest] "/manifest"
    source = ""
    while ((getline line < file) > 0) { sub(/\r$/, "", line); source = source line "\n" }
    close(file)
    task_path = path
    manifests[digest] = manifest_root = node("table", "")
    parse_manifest()
  }
  roots[app] = root = manifests[digest]
  print "app", app, path

  # Keep every pin's occurrence, with its parent after its dependencies
  ancestors[digest] = 1
  pins = member(root, "apps")
  for (i = 1; i <= sizes[pins]; i++) {
    pin = entries[pins, i]
    name = local_name(pin, path)
    child = load_app(field(pin, "sha256"), path == "." ? name : path "/" name, depth + 1)
    children[app, ++child_count[app]] = child
    child_named[app, name] = child
    bindings[child] = member(pin, "inputs")
    parents[child] = app
    local_names[child] = name
  }
  ancestors[digest] = 0
  order[++order_count] = app
  return app
}

# Resolve each binding to a literal, a parent's input or a sibling's output.
function bind_inputs(app,    inputs, i, name, value, parent, ref, parts, producer) {
  parent = parents[app]
  inputs = member(roots[app], "inputs")
  for (i = 1; i <= sizes[bindings[app]]; i++) {
    name = local_name(entries[bindings[app], i], paths[app])
    if (!member(inputs, name)) fail(paths[app], "binding names undeclared input " name)
  }
  for (i = 1; i <= sizes[inputs]; i++) {
    name = local_name(entries[inputs, i], paths[app])
    if (!parent) { print "answer", app, name; settled[app, name] = 1; continue }
    value = member(bindings[app], name)
    if (!value) fail(paths[app], "unbound input " name)
    if (node_type[value] != "table") {
      source_kind[app, name] = "literal"
      literals[app, name] = value
    } else if (member(value, "input")) {
      ref = field(value, "input")
      if (!member(member(roots[parent], "inputs"), ref))
        fail(paths[app], "undeclared parent input " ref)
      source_kind[app, name] = "inputs"
      source_app[app, name] = parent
      source_name[app, name] = ref
    } else {
      ref = field(value, "output")
      if (split(ref, parts, "/") != 2 || !(producer = child_named[parent, parts[1]]))
        fail(paths[app], "undeclared output source " ref)
      if (!member(member(roots[producer], "outputs"), parts[2]))
        fail(paths[app], "undeclared output " ref)
      source_kind[app, name] = "outputs"
      source_app[app, name] = producer
      source_name[app, name] = parts[2]
    }
  }
}

# Join literal picks with escaped newlines, preserving every byte.
function literal_bytes(value,    i, result) {
  if (node_type[value] != "array") return node_value[value]
  for (i = 1; i <= sizes[value]; i++)
    result = result (i == 1 ? "" : "\\0012") node_value[entries[value, i]]
  return result
}

# Settle an input once its source is ready, even if its parent cannot run yet.
function settle_input(app, name,    kind, producer, ref, declaration) {
  if (settled[app, name]) return 1
  kind = source_kind[app, name]
  producer = source_app[app, name]
  ref = source_name[app, name]
  if (kind == "outputs" && !succeeded[producer]) return 0
  if (kind == "inputs" && !settle_input(producer, ref)) return 0
  if (kind == "literal") print "literal", app, name, literal_bytes(literals[app, name])
  else print "copy", app, name, producer, kind, ref

  # A receiving multiple choice orders picks by its own declaration
  declaration = member(member(roots[app], "inputs"), name)
  if (parents[app] && node_value[member(declaration, "multiple")] == "true")
    print "choices", app, name, literal_bytes(member(declaration, "choices"))
  settled[app, name] = 1
  return 1
}

# Reject grant paths that could escape the temporary mount tree.
function relative_path(value) {
  return value ~ /^v1\// && value !~ /[[:space:][:cntrl:]]/ &&
         value !~ /(^|\/)\.\.?($|\/)/ && value !~ /\/\/|\/$/
}

# Plan grants for every occurrence before starting any processing pass.
function prepare_grants(app,    reads, kinds, k, grants, i, grant, j, directory) {
  reads = member(roots[app], "reads")
  split("paths optional", kinds, " ")
  for (k = 1; k <= 2; k++) {
    grants = member(reads, kinds[k])
    for (i = 1; i <= sizes[grants]; i++) {
      grant = plain(node_value[entries[grants, i]])
      if (!relative_path(grant)) fail(paths[app], "unsupported grant path")
      if (k == 2 && ENVIRON["OPTIONAL"] == "off") continue
      for (j = 1; j <= directory_count; j++) {
        directory = directories[j]
        if (directory == grant || index(directory, grant "/") == 1)
          print "directory", app, directory
      }
      print "grant", app, kinds[k], grant, paths[app]
    }
  }
  print "freeze-grants", app
}

END {
  OFS = "\t"
  if (task == "pins") {
    manifest_root = node("table", "")
    parse_manifest()
    pins = member(manifest_root, "apps")
    for (i = 1; i <= sizes[pins]; i++) {
      pin = entries[pins, i]
      print local_name(pin, task_path), field(pin, "sha256")
    }
  } else if (task == "plan") {
    stage = ENVIRON["RUNNER_STAGE"]
    file = stage "/modules/index"
    while ((getline line < file) > 0) {
      split(line, fields, " ")
      if (!modules[fields[1]]) modules[fields[1]] = fields[2]
      if (fields[2] == 1) main_digest = fields[1]
    }
    close(file)
    load_app(main_digest, ".", 0)
    file = ENVIRON["RUNNER_FIXTURES"] "/directories"
    while ((getline line < file) > 0)
      if (relative_path(line)) directories[++directory_count] = line
    close(file)
    for (app = 1; app <= app_count; app++) {
      bind_inputs(app)
      prepare_grants(app)
    }

    # Compute the first-ready order once, using only the dependency graph
    remaining = order_count
    while (remaining) {
      ready = 0
      for (i = 1; i <= order_count; i++) {
        app = order[i]
        if (succeeded[app]) continue
        ready = 1
        for (j = 1; j <= child_count[app]; j++)
          if (!succeeded[children[app, j]]) ready = 0
        inputs = member(roots[app], "inputs")
        for (j = 1; j <= sizes[inputs]; j++)
          if (!settle_input(app, plain(node_name[entries[inputs, j]]))) ready = 0
        if (ready) break
      }
      if (!ready) fail(paths[app], "cycle in input bindings")
      if (sizes[inputs]) print "freeze-inputs", app
      for (j = 1; j <= child_count[app]; j++) {
        child = children[app, j]
        if (sizes[member(roots[child], "outputs")])
          print "dependency", app, child, local_names[child]
      }
      outputs = member(roots[app], "outputs")
      for (j = 1; j <= sizes[outputs]; j++)
        print "output", app, local_name(entries[outputs, j], paths[app])
      print "processing", app, module_ids[app], paths[app]
      succeeded[app] = 1
      remaining--
    }
  }
}
