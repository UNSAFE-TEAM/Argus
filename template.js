// Preferred rule template: consume an existing sensor.
// Rules, profiles, and behavior modules should use this form whenever a sensor exists.
(() => {
  const Agent = globalThis.AgentV1;
  const ArgusSensors = globalThis.ArgusSensorsV1;
  const TAG = "category";
  const SENSOR_NAME = "Module.ApiName";

  ArgusSensors.use(SENSOR_NAME, {
    name: "category.rule_name",

    match(ctx) {
      // Return true only when this rule needs to act.
      // Example: return Agent.containsAny(ctx.path, ["vmware"]);
      return false;
    },

    apply(ctx) {
      // Mutate through ctx helper methods exposed by the sensor.
      // Example: ctx.notFound();
      // Example: ctx.replaceString("C:\\Windows\\System32\\version.dll");

      Agent.collect(TAG, ctx.moduleName, ctx.apiName, ctx.caller, [], []);
      Agent.triggered(TAG, ctx.moduleName, ctx.apiName, ctx.caller, {
        original: {},
        current: {},
      });
    },
  });
})();

/*
Sensor template: define one API capability. Do not call .install().
ArgusSensors.use(...) installs sensors on demand.

(() => {
  const Agent = globalThis.AgentV1;
  const ArgusSensors = globalThis.ArgusSensorsV1;
  const SENSOR_NAME = "Module.ApiName";
  const TAG = "sensor";

  const API_HOOKS = [
    { moduleName: "module.dll", apiName: "ApiNameA", wide: false },
    { moduleName: "module.dll", apiName: "ApiNameW", wide: true },
  ];

  ArgusSensors.define(SENSOR_NAME, (sensor) => {
    for (const hook of API_HOOKS) {
      Agent.whenModuleLoaded(hook.moduleName, () => {
        Agent.attachApi(TAG, hook.moduleName, hook.apiName, () => ({
          onEnter(args) {
            this.ctx = {
              sensor: SENSOR_NAME,
              moduleName: hook.moduleName,
              apiName: hook.apiName,
              wide: hook.wide,
              caller: this.returnAddress.toString(),
              retval: null,
            };
          },

          onLeave(retval) {
            this.ctx.retval = retval;
            sensor.emit(this.ctx);
          },
        }));
      });
    }
  });
})();

Direct hook template: use only when no sensor exists yet.

(() => {
  const Agent = globalThis.AgentV1;
  const TAG = "category";
  const MODULE_NAME = "module.dll";
  const API_NAME = "ApiName";

  const ARG_SPEC = [
    // { index: 0, name: "arg0" },
  ];

  function install() {
    Agent.attachApi(TAG, MODULE_NAME, API_NAME, () => ({
      onEnter(args) {
        this.caller = this.returnAddress;
        Agent.collect(TAG, MODULE_NAME, API_NAME, this.caller.toString(), args, ARG_SPEC);
      },

      onLeave(retval) {
        Agent.triggered(TAG, MODULE_NAME, API_NAME, this.caller.toString(), {
          original: { return: retval.toString() },
          current: { return: retval.toString() },
        });
      },
    }));
  }

  Agent.safeCall(TAG, () => {
    Agent.whenModuleLoaded(MODULE_NAME, install);
  });
})();
*/
