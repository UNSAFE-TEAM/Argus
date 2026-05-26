(() => {
  const TAG = "category";
  const API_NAME = "ApiName";
  const MODULE_NAME = "module.dll";

  const ARG_SPEC = [
    // { index: 0, name: "arg0" },
    // { index: 1, name: "arg1" },
  ];

  Agent.safeCall(TAG, () => {
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

    Agent.register(TAG, MODULE_NAME, API_NAME);
  });
})();
