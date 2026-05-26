(() => {
  const TAG = "anti_debug";
  const API_NAME = "IsDebuggerPresent";
  const MODULE_NAME = "kernel32.dll";

  Agent.safeCall(TAG, () => {
    const addr = Agent.getExport(MODULE_NAME, API_NAME);

    Interceptor.attach(addr, {
      onLeave(retval) {
        const original = retval.toInt32();

        retval.replace(0);

        Agent.triggered(TAG, {
          apiName: API_NAME,
          moduleName: MODULE_NAME,
          originalReturn: original,
          patched: true,
          patchedReturn: 0,
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
