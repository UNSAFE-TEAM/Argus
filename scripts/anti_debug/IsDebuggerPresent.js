(() => {
  const TAG = "anti_debug";
  const API_NAME = "IsDebuggerPresent";
  const MODULE_NAME = "kernel32.dll";
  const ARG_SPEC = [];

  let installed = false;

  function install() {
    if (installed) {
      return;
    }

    const addr = Agent.getExport(MODULE_NAME, API_NAME);

    Interceptor.attach(addr, {
      onEnter(_args) {
        this.caller = this.returnAddress;

        Agent.collect(
          TAG,
          MODULE_NAME,
          API_NAME,
          this.caller.toString(),
          _args,
          ARG_SPEC,
        );
      },

      onLeave(retval) {
        const original = retval.toInt32();

        retval.replace(0);

        Agent.triggered(TAG, MODULE_NAME, API_NAME, this.caller.toString(), {
          original: { return: String(original) },
          current: { return: "0" },
        });
      },
    });

    installed = true;
    Agent.register(TAG, MODULE_NAME, API_NAME);
  }

  Agent.safeCall(TAG, () => {
    Agent.whenModuleLoaded(MODULE_NAME, install);
  });
})();
