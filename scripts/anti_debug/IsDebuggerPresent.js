(() => {
  const TAG = "anti_debug";
  const API_NAME = "IsDebuggerPresent";
  const MODULE_NAME = "kernel32.dll";

  Agent.safeCall(TAG, () => {
    const addr = Agent.getExport(MODULE_NAME, API_NAME);

    Interceptor.attach(addr, {
      onEnter(_args) {
        this.caller = this.returnAddress;

        Agent.collect(
          TAG,
          MODULE_NAME,
          API_NAME,
          this.caller.toString(),
          [],
          [],
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

    Agent.register(TAG, MODULE_NAME, API_NAME);
  });
})();
