(() => {
  const TAG = "category";
  const API_NAME = "ApiName";
  const MODULE_NAME = "module.dll";

  const ARG_SPEC = [
    // { index: 0, name: "arg0" },
    // { index: 1, name: "arg1" },
  ];

  let installed = false;

  function install() {
    if (installed) {
      return;
    }

    const addr = Agent.getExport(MODULE_NAME, API_NAME);

    Interceptor.attach(addr, {
      onEnter(args) {
        this.caller = this.returnAddress;

        Agent.collect(
          TAG,
          MODULE_NAME,
          API_NAME,
          this.caller.toString(),
          args,
          ARG_SPEC,
        );

        // Optional parsed fields
        // this.path = Agent.readUtf16(args[0]);
      },

      onLeave(retval) {
        Agent.triggered(TAG, MODULE_NAME, API_NAME, this.caller.toString(), {
          original: { return: retval.toString() },
          current: { return: retval.toString() },

          // Optional parsed fields
          // path: this.path,
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
