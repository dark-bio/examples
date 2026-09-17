// Initialize an isolated interpreter and execute the frozen app.
#include <Python.h>
#include "frozen.h"

int main(int argc, char **argv) {
  PyConfig config;
  PyConfig_InitIsolatedConfig(&config);
  config.site_import = 0;
  config.write_bytecode = 0;
  config.parse_argv = 0;
  config.install_signal_handlers = 0;
  config.module_search_paths_set = 1;
  config.use_frozen_modules = 1;
  config.pathconfig_warnings = 0;
  config.use_hash_seed = 1;
  config.hash_seed = 0;
  PyImport_FrozenModules = app_modules;

  PyStatus status = PyConfig_SetBytesArgv(&config, argc, argv);
  if (!PyStatus_Exception(status)) {
    status = PyConfig_SetString(&config, &config.program_name, L"ark-python");
  }
  if (!PyStatus_Exception(status)) status = Py_InitializeFromConfig(&config);
  PyConfig_Clear(&config);
  if (PyStatus_Exception(status)) Py_ExitStatusException(status);

  int result = PyImport_ImportFrozenModule("__main__") > 0 ? 0 : 1;
  if (result) PyErr_Print();
  if (Py_FinalizeEx() < 0) result = 120;
  return result;
}
