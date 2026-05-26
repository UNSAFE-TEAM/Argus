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
        Agent.collect(TAG, args, ARG_SPEC, {
          apiName: API_NAME,
          moduleName: MODULE_NAME,
        });

        // Optional parsed fields
        // this.path = Agent.readUtf16(args[0]);
      },

      onLeave(retval) {
        Agent.triggered(TAG, {
          apiName: API_NAME,
          moduleName: MODULE_NAME,

          // Optional parsed fields
          // path: this.path,
        });
      },
    });

    Agent.register(TAG, {
      apiName: API_NAME,
      moduleName: MODULE_NAME,
      address: addr.toString(),
    });
  });
})();
