// Start an isolated CPython and run the app's frozen __main__ module. The app
// carries every module it imports, so the interpreter needs no filesystem of
// its own and reads nothing but the data the Ark mounts.
#include <Python.h>
#include "frozen.h"

// Py_FinalizeEx reports a failure to flush the standard streams, which CPython
// itself reports to the shell as exit code 120.
#define FLUSH_FAILED 120

static PyStatus configure(PyConfig *config, int argc, char **argv) {
  PyConfig_InitIsolatedConfig(config);
  config->site_import = 0;               // there are no installed packages
  config->write_bytecode = 0;            // and nowhere to cache any
  config->parse_argv = 0;                // the app reads sys.argv itself
  config->install_signal_handlers = 0;
  config->module_search_paths_set = 1;   // an empty search path, frozen only
  config->use_frozen_modules = 1;
  config->pathconfig_warnings = 0;
  config->use_hash_seed = 1;             // a deterministic sandbox, so a fixed
  config->hash_seed = 0;                 // seed rather than a random one

  PyStatus status = PyConfig_SetBytesArgv(config, argc, argv);
  if (PyStatus_Exception(status)) {
    return status;
  }
  return PyConfig_SetString(config, &config->program_name, L"ark-python");
}

int main(int argc, char **argv) {
  PyImport_FrozenModules = app_modules;

  PyConfig config;
  PyStatus status = configure(&config, argc, argv);
  if (!PyStatus_Exception(status)) {
    status = Py_InitializeFromConfig(&config);
  }
  PyConfig_Clear(&config);
  if (PyStatus_Exception(status)) {
    Py_ExitStatusException(status);
  }

  // Importing the frozen module runs the app. It answers 1 once the module has
  // run, 0 when there is no such module, and -1 for an exception, which is also
  // how the app exits with a failure.
  int result = PyImport_ImportFrozenModule("__main__") > 0 ? 0 : 1;
  if (result && PyErr_Occurred()) {
    PyErr_Print();
  }
  if (Py_FinalizeEx() < 0) {
    result = FLUSH_FAILED;
  }
  return result;
}
