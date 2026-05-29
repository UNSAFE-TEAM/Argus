(() => {
  const TAG = "category";
  const MODULE_NAME = "module.dll";
  const API_NAME = "ApiName";

  const ARG_SPEC = [
    // { index: 0, name: "arg0" },
    // { index: 1, name: "arg1" },
  ];

  function install() {
    Agent.attachApi(TAG, MODULE_NAME, API_NAME, () => ({
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
    }));
  }

  Agent.safeCall(TAG, () => {
    Agent.whenModuleLoaded(MODULE_NAME, install);
  });
})();
